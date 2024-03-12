package main

import (
	"fmt"
	"io"
	"net"
	"os"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/shirou/gopsutil/v3/cpu"
	"github.com/shirou/gopsutil/v3/mem"
)

type connectionManager struct {
	connections []net.Conn
	mux         sync.Mutex
}

type SystemUsage struct {
	cpu float64
	mem float64
}

// TODO: Find a way to not need to sleep for time.Second
func refresh(sys *SystemUsage) {
	v, _ := mem.VirtualMemory()
	c, _ := cpu.Percent(time.Second, false)
	sys.cpu = c[0]
	sys.mem = v.UsedPercent

}

func show(sys *SystemUsage, ctx string) {
	fmt.Println(ctx)
	fmt.Printf("Memory usage: %.2f%%\n", sys.mem)
	fmt.Printf("CPU usage: %.2f%%\n", sys.cpu)
}

func main() {
	var wg sync.WaitGroup

	var sys SystemUsage
	refresh(&sys)
	show(&sys, "Start of program")

	var ip = get_ip(&sys)

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
	refresh(&sys)
	show(&sys, "After binding to socket")
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
		go handleConnection(conn, &wg, cm, &sys)
		refresh(&sys)
		show(&sys, fmt.Sprintf("After accepting new connection (%d total connection(s))", len(cm.connections)))
	}
}

func handleConnection(conn net.Conn, wg *sync.WaitGroup, cm *connectionManager, sys *SystemUsage) {
	refresh(sys)
	show(sys, "After spawning goroutine")

	// reverse order because defer works opposite
	defer show(sys, fmt.Sprintf("After closing connection (%d total connection(s))", len(cm.connections)))
	defer refresh(sys)

	defer wg.Done()

	fmt.Println("Accepted connection from:", conn.RemoteAddr())

	buffer := make([]byte, 256)

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
			refresh(sys)
			show(sys, "After receiving data")

			fmt.Printf("Sending %d bytes from %s\n", len(out), conn.RemoteAddr().String())
			for _, connection := range cm.connections {
				if conn.RemoteAddr() != connection.RemoteAddr() {
					_, err = connection.Write(out)
					if err != nil {
						fmt.Println("Error writing to client:", err)
					}
				}
			}
			refresh(sys)
			show(sys, fmt.Sprintf("After sending data to %d client(s)", len(cm.connections)-1))
		}
	}
}

func generateOutput(n int, buffer []byte, conn net.Conn) ([]byte, bool) {
	if n == 1 && buffer[n-1] == 10 {
		return []byte(""), false
	}

	var output []byte
	addr := []byte(conn.RemoteAddr().String() + ": ") // read so lock not needed

	output = append(output, addr...)
	if buffer[n] == 10 || buffer[n-1] == 10 {
		output = append(output, buffer[:n-1]...)
	} else {
		output = append(output, buffer[:n]...)
	}
	return output, true
}

func get_ip(sys *SystemUsage) string {
	refresh(sys)
	show(sys, "Before getting IP from args")
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
		fmt.Println("No arguments have been passed, using 127.0.0.1:6969")
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
	refresh(sys)
	show(sys, "After getting IP from args (and parsing)")
	return string(strings.Join(ip, ""))

}
