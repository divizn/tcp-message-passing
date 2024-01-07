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

func main() {
	var wg sync.WaitGroup

	var ip = get_ip()
	fmt.Println(ip)
	var connections []net.Conn

	stream, err := net.Listen("tcp", ip)
	if err != nil {
		fmt.Println("Error initialising server:", err)
		return
	}
	fmt.Println("Listening at", ip)
	defer stream.Close()

	for {
		conn, err := stream.Accept()
		if err != nil {
			fmt.Println("Error accepting a connection:", err)
			continue
		}
		connections = append(connections, conn)
		wg.Add(1)
		go handleConnection(conn, &wg, &connections)

	}
}

func handleConnection(conn net.Conn, wg *sync.WaitGroup, connections *[]net.Conn) {
	defer wg.Done()
	fmt.Println("Accepted connection from:", conn.RemoteAddr())

	buffer := make([]byte, 256)

	for {
		n, err := conn.Read(buffer)
		if err != nil {
			if err == io.EOF {
				fmt.Println("Connection has been closed by", conn.RemoteAddr())
				// for i, connection := range *connections {
				// 	if connection.RemoteAddr() != conn.RemoteAddr() {
				// 		*connections = append((*connections)[:i], (*connections)[i+1:]...)
				// 	}
				// }
				// TODO: Remove conn from connections[]
				return
			}
			fmt.Println("Error on connection:", conn.RemoteAddr(), ",", err, ". Closing connection.")
			// TODO: Remove conn from connections[]
			return
		}
		fmt.Println("Received", buffer[:n], "from", conn.RemoteAddr())
		if n != 0 {
			out, valid := generateOutput(n, buffer, conn)
			if !valid {
				continue
			}
			fmt.Println("Sending", out, "from", conn.RemoteAddr())
			for _, connection := range *connections {
				if conn.RemoteAddr() != connection.RemoteAddr() {
					_, err = connection.Write(out)
					if err != nil {
						fmt.Println("Error writing to client:", err)
					}
				}
			}
		}
	}
}

func generateOutput(n int, buffer []byte, conn net.Conn) ([]byte, bool) {
	fmt.Println(buffer[n], "or", buffer[n-1])
	if n == 1 && buffer[n] == 10 {
		return []byte(""), false
	}

	var output []byte
	addr := []byte(conn.RemoteAddr().String() + ": ")

	output = append(output, addr...)
	if buffer[n] == 10 || buffer[n-1] == 10 {
		output = append(output, buffer[:n-1]...)
	} else {
		output = append(output, buffer[:n]...)
	}
	return output, true
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
		ip = append(ip, "127.0.0.1:6969")
	}

	if len(args) > 1 {
		if len(args) > 1 {
			port, err := strconv.Atoi(args[1])
			if err == nil && port >= 2000 && port <= 65535 { // 2000 because others are reserved (prob)
				ip = append(ip, ":"+args[1])
			} else {
				fmt.Println("Invalid port number provided, using 6969")
				ip = append(ip, ":6969")
			}
		} else {
			fmt.Println("No port number provided, using 6969")
			ip = append(ip, ":6969")
		}
	}

	return string(strings.Join(ip, ""))

}
