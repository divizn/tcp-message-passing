package main

import (
	"fmt"
	"io"
	"net"
	"os"
	"strconv"
	"strings"
	"sync"
)

type connectionManager struct {
	connections []net.Conn
	mux         sync.Mutex
}

func main() {
	var wg sync.WaitGroup

	var ip = get_ip()

	cm := &connectionManager{}

	stream, err := net.Listen("tcp", ip)
	if err != nil {
		curr_ip := strings.Split(ip, ":")
		ip = ""
		ip += curr_ip[0] + ":"
		port, err := strconv.Atoi(curr_ip[1])
		if err != nil {
			fmt.Println("Port is invalid:", err)
		}
		port += 1
		ip += strconv.Itoa(port)
		stream, err = net.Listen("tcp", ip)
		if err != nil {
			fmt.Println("Error initialising server, maybe try another port\n", err)
			return
		}
	}
	fmt.Println("Listening at", ip)
	defer stream.Close()

	for {
		conn, err := stream.Accept()
		if err != nil {
			fmt.Println("Error accepting a connection:", err)
			continue
		}
		cm.mux.Lock()
		cm.connections = append(cm.connections, conn)
		wg.Add(1)
		cm.mux.Unlock()
		go handleConnection(conn, &wg, cm)
	}
}

func handleConnection(conn net.Conn, wg *sync.WaitGroup, cm *connectionManager) {
	defer wg.Done()
	fmt.Println("Accepted connection from:", conn.RemoteAddr())

	buffer := make([]byte, 1024)

	for {
		n, err := conn.Read(buffer)
		if err != nil {
			if err == io.EOF {
				fmt.Println("Connection has been closed by", conn.RemoteAddr())
			} else {
				fmt.Println("Error on connection:", conn.RemoteAddr(), ",", err, ". Closing connection.")
			}
			var updatedConnections []net.Conn
			cm.mux.Lock()
			for _, connection := range cm.connections {
				if connection.RemoteAddr() != conn.RemoteAddr() {
					updatedConnections = append(updatedConnections, connection)
				}
			}
			cm.connections = updatedConnections
			cm.mux.Unlock()
			return
		}
		fmt.Printf("Received %d bytes from %s\n", n, conn.RemoteAddr().String())
		if n != 0 {
			out, valid := generateOutput(n, buffer, conn)
			if !valid {
				continue
			}
			fmt.Printf("Sending %d bytes from %s\n", len(out), conn.RemoteAddr().String())
			for _, connection := range cm.connections {
				if conn.RemoteAddr() != connection.RemoteAddr() {
					_, err = connection.Write([]byte(out))
					if err != nil {
						fmt.Println("Error writing to client:", err)
					}
				}
			}
		}
	}
}

func generateOutput(n int, buffer []byte, conn net.Conn) (string, bool) {
	if n == 1 && buffer[0] == 10 {
		return "", false
	}

	var output []byte
	addr := []byte(conn.RemoteAddr().String() + ": ") // read so lock not needed

	output = append(output, addr...)
	if buffer[n] == 10 || buffer[n-1] == 10 {
		output = append(output, buffer[:n-1]...)
	} else {
		output = append(output, buffer[:n]...)
	}
	return string(output), true
}

func get_ip() string {
	var ip []string // socket not ip
	var args = os.Args[1:]

	if len(args) > 0 {
		addr := net.ParseIP(args[0])
		if addr != nil && addr.To4() != nil {
			ip = append(ip, addr.String())
		} else {
			fmt.Println("Invalid IPv4 address provided, using 127.0.0.1")
			ip = append(ip, "127.0.0.1")
		}
	} else {
		fmt.Println("No arguments have been passed, using 127.0.0.1:8000")
		ip = append(ip, "127.0.0.1:8000")
	}

	if len(args) > 1 {
		if len(args) > 1 {
			port, err := strconv.Atoi(args[1])
			if err == nil && port >= 2000 && port <= 65535 { // 2000 because others are reserved (prob)
				ip = append(ip, ":"+args[1])
			} else {
				fmt.Println("Invalid port number provided, using 8000")
				ip = append(ip, ":8000")
			}
		} else {
			fmt.Println("No port number provided, using 8000")
			ip = append(ip, ":8000")
		}
	}
	return string(strings.Join(ip, ""))

}
