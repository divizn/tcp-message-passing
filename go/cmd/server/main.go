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
