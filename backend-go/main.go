package main

import (
	"fmt"
	"github.com/akamensky/argparse"
	"os"
	"pixelflut/network"
	"pixelflut/protocol"
	"sync"
	"time"
)

var tcpPort *string
var websocketPort *string
var udpPort *string
var xSize uint
var ySize uint

func main() {
	parseArguments()
	pixmap := protocol.NewPixmap(xSize, ySize, []byte {0, 0, 0})
	fmt.Printf("Initialized new pixmap of size %vx%v\n", xSize, ySize)

	waitGroup := &sync.WaitGroup{}

	if *tcpPort != "" {
		waitGroup.Add(1)
		go network.StartTcpServer(*tcpPort, pixmap, waitGroup)
	}
	if *websocketPort != "" {
		waitGroup.Add(1)
		go network.StartWebsocketServer(*websocketPort, pixmap, waitGroup)
	}
	if *udpPort != "" {
		waitGroup.Add(1)
		go network.StartUdpServer(*udpPort, pixmap, waitGroup)
	}

	stateTicker := time.NewTicker(100 * time.Millisecond)
	go pixmapStateWorker(stateTicker.C, pixmap, waitGroup)

	waitGroup.Wait()
}

func pixmapStateWorker(tick <-chan time.Time, pixmap *protocol.Pixmap, waitGroup *sync.WaitGroup) {
	defer waitGroup.Done()

	for {
		<-tick
		pixmap.CalculateStates()
	}
}

func parseArguments() {
	parser := argparse.NewParser("pixelflut", "a pixel drawing game for programmers (server)")

	tcpPort = parser.String("t", "tcp", &argparse.Options{
		Help: "Listen for TCP connections on the specified port",
	})
	websocketPort = parser.String("w", "websocket", &argparse.Options{
		Help: "Listen for Websocket connections on the specified port",
	})
	udpPort = parser.String("u", "udp", &argparse.Options{
		Help: "Listen fo UDP messages on the specified port",
	})
	xSizeInt := parser.Int("x", "xSize", &argparse.Options{
		Required: false,
		Help:     "Size of the canvas in x dimension",
		Default:  800,
	})
	ySizeInt := parser.Int("y", "ySize", &argparse.Options{
		Required: false,
		Help:     "Size of the canvas in y dimension",
		Default:  600,
	})

	if err := parser.Parse(os.Args); err != nil {
		fmt.Println(parser.Usage(err))
		os.Exit(1)
	}

	if *xSizeInt < 0 {
		fmt.Println(parser.Usage("xSize cannot be smaller than 0"))
		os.Exit(1)
	} else {
		xSize = uint(*xSizeInt)
	}

	if *ySizeInt < 0 {
		fmt.Println(parser.Usage("ySize cannot be smaller than 0"))
		os.Exit(1)
	} else {
		ySize = uint(*ySizeInt)
	}
}
