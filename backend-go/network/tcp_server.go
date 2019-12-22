package network

import (
	"bytes"
	"fmt"
	"net"
	"os"
	"pixelflut.backend/protocol"
)

type TcpHandler struct {
	pixmap *protocol.Pixmap
}

func readLine(connection net.Conn) (string, error) {
	largeBuffer := make([]byte, 0)
	smallBuffer := make([]byte, 128)
	for {
		if _, err := connection.Read(smallBuffer); err != nil {
			return "", err
		} else {
			if i := bytes.IndexRune(smallBuffer, '\n'); i == -1 {
				largeBuffer = append(largeBuffer, smallBuffer...)
			} else {
				return string(append(largeBuffer, smallBuffer[:i]...)), nil
			}
		}
	}
}

func handleConnection(connection net.Conn, pixmap *protocol.Pixmap) {
	fmt.Println("[TCP] New connection from", connection.RemoteAddr().String())

	for {
		if input, err := readLine(connection); err != nil {
			fmt.Printf("[TCP] Error: %v\n", err)
			_ = connection.Close()
			return
		} else {
			fmt.Printf("[TCP] Received: %v\n", input)
			response := protocol.ParseAndHandleInput(input, pixmap)
			fmt.Printf("[TCP] Response: %v\n", response)
			connection.Write([]byte(response))
		}
	}
}

func Start(port string, pixmap *protocol.Pixmap) {
	if ln, err := net.Listen("tcp", net.JoinHostPort("", port)); err != nil {
		fmt.Printf("[TCP] Could not start tcp listener on port %v: %v", port, err)
		os.Exit(1)
	} else {
		fmt.Println("[TCP] Accepting connections on port", port)
		for {
			if conn, err := ln.Accept(); err != nil {
				fmt.Printf("[TCP] Could not accept new connection: %v\n", err)
			} else {
				go handleConnection(conn, pixmap)
			}
		}
	}
}
