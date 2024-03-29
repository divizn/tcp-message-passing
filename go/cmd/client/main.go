package main

import (
	"bufio"
	"fmt"
	"io"
	"net"
	"os"
	"sync"
)

func main() {
	var ip string

	if len(os.Args) > 1 {
		ip = os.Args[1]
	} else {
		fmt.Println("No IP provided")
		return
	}

	var wg sync.WaitGroup
	wg.Add(2)

	conn, err := net.Dial("tcp", ip)
	if err != nil {
		fmt.Println("Error connecting to server: ", err)
		return
	}
	defer conn.Close() // close connection when program terminated.

	fmt.Println("Connected to server at:", conn.RemoteAddr())

	go readStream(conn, &wg)
	go handleInput(conn, &wg)
	wg.Wait()
}

func handleInput(conn net.Conn, wg *sync.WaitGroup) {
	defer wg.Done()

	ioBuffer := bufio.NewReader(os.Stdin)
	for {
		inp, err := ioBuffer.ReadString('\n')
		if err != nil {
			continue
		}
		_, err = conn.Write([]byte(inp))
		if err != nil {
			fmt.Println("Error writing to server", err)
		}
	}
}

func readStream(conn net.Conn, wg *sync.WaitGroup) {
	defer wg.Done()
	buff := make([]byte, 256)

	for {
		n, err := conn.Read(buff)
		if err != nil {
			if err == io.EOF {
				fmt.Println("Server closed connection")
				return
			}
			fmt.Println("Error reading message from server:", err)
			return
		}
		if n != 0 {
			receivedMessage := string(buff[:n])
			fmt.Println(receivedMessage)
		}

	}

}
