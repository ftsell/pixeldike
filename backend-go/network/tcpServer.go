package network

import (
	"fmt"
	"github.com/ftsell/pixelflut/backend-go/protocol"
	"github.com/ftsell/pixelflut/backend-go/util"
	"net"
	"os"
	"sync"
)

type TcpHandler struct {
	pixmap *protocol.Pixmap
}

func handleTcpConnection(connection net.Conn, pixmap *protocol.Pixmap) {
	fmt.Println("[TCP] New connection from", connection.RemoteAddr())

	for {
		if input, err := util.ReadLine(connection); err != nil {
			fmt.Printf("[TCP] Error reading: %v\n", err)
			_ = connection.Close()
			return
		} else {
			go func() {response := protocol.ParseAndHandleInput(input, pixmap)
				if numWritten, err := connection.Write([]byte(response)); err != nil {
					fmt.Printf("[TCP] Error writing: %v\n", err)
				} else if numWritten != len(response) {
					fmt.Printf("[TCP] Wrote incorrect byte amount: %v out of %v", numWritten, len(response))
				}
			}()
		}
	}
}

func StartTcpServer(port string, pixmap *protocol.Pixmap, waitGroup *sync.WaitGroup) {
	defer waitGroup.Done()

	if ln, err := net.Listen("tcp", net.JoinHostPort("", port)); err != nil {
		fmt.Printf("[TCP] Could not start tcp listener on port %v: %v", port, err)
		os.Exit(1)
	} else {
		defer ln.Close()
		fmt.Println("[TCP] Accepting connections on port", port)

		for {
			if conn, err := ln.Accept(); err != nil {
				fmt.Printf("[TCP] Could not accept new connection: %v\n", err)
			} else {
				go handleTcpConnection(conn, pixmap)
			}
		}
	}
}
