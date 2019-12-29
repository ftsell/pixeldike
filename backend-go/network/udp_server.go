package network

import (
	"fmt"
	"net"
	"os"
	"pixelflut/protocol"
	"sync"
)

func StartUdpServer(port string, pixmap *protocol.Pixmap, waitGroup *sync.WaitGroup)  {
	defer waitGroup.Done();
	
	addr, _ := net.ResolveUDPAddr("udp", net.JoinHostPort("", port))
	if ln, err := net.ListenUDP("udp", addr); err != nil {
		fmt.Printf("[UDP] Could not start udp listener on port %v: %v\n", port, err)
		os.Exit(1)
	} else {
		defer ln.Close()
		fmt.Printf("[UDP] Listening for datagram packets on port %v\n", port)

		for {
			if line, err := readLine(ln); err != nil {
				fmt.Printf("[UDP] Could not receive packet: %v\n", err)
			} else {
				go protocol.ParseAndHandleInput(line, pixmap)
			}
		}
	}
}
