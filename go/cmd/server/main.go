package main

import (
	"fmt"
	"io"
	"net"
	"sync"
)

// TODO: Get IP and port from args

func main() {
	var wg sync.WaitGroup

	var connections []net.Conn

	stream, err := net.Listen("tcp", "127.0.0.1:6969")
	if err != nil {
		fmt.Println("Error initialising server:", err)
		return
	}
	fmt.Println("Listening at 127.0.0.1:6969")

	// buffer := bufio.NewReader(stream)

	defer stream.Close()

	for {
		conn, err := stream.Accept()
		if err != nil {
			fmt.Println("Error accepting a connection:", err)
			continue
		}
		connections = append(connections, conn)
		wg.Add(1)
		go handleConnection(conn, &wg, connections)

	}
}

func handleConnection(conn net.Conn, wg *sync.WaitGroup, connections []net.Conn) {
	defer wg.Done()
	fmt.Println(conn, connections)

	buffer := make([]byte, 256)

	for {
		n, err := conn.Read(buffer)
		if err != nil {
			if err == io.EOF {
				fmt.Println("Connection has been closed by", conn.RemoteAddr())
				// TODO: Remove conn from connections[]
				return
			}
			fmt.Println("Error on connection:", conn.RemoteAddr(), ",", err, ". Closing connection.")
			// TODO: Remove conn from connections[]
			return
		}
		fmt.Println("Received", buffer[:n], "from", conn.RemoteAddr())
		if n != 0 {
			var out []byte
			if buffer[n] == 10 {
				out = buffer[:n-1]
			} else {
				out = buffer[:n]
			}
			fmt.Println("Sending", out, "from", conn.RemoteAddr())
			for _, connection := range connections {
				if conn.RemoteAddr() != connection.RemoteAddr() {
					_, err = conn.Write(out)
					if err != nil {
						fmt.Println("Error writing to client:", err)
					}
				}
			}
		}
	}
}
