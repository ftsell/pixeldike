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

var argTcpPort *string
var argWebsocketPort *string
var argUdpPort *string
var argSnapshotFile *os.File
var argNewPixmap *bool
var argPixmapWidth uint
var argPixmapHeight uint

func main() {
	parseArguments()
	pixmap := initPixmap()

	waitGroup := &sync.WaitGroup{}

	if *argTcpPort != "" {
		waitGroup.Add(1)
		go network.StartTcpServer(*argTcpPort, pixmap, waitGroup)
	}
	if *argWebsocketPort != "" {
		waitGroup.Add(1)
		go network.StartWebsocketServer(*argWebsocketPort, pixmap, waitGroup)
	}
	if *argUdpPort != "" {
		waitGroup.Add(1)
		go network.StartUdpServer(*argUdpPort, pixmap, waitGroup)
	}

	stateTicker := time.NewTicker(100 * time.Millisecond)
	go pixmapStateWorker(stateTicker.C, pixmap)

	if *argSnapshotFile != (os.File{}) {
		snapshotTicker := time.NewTicker(10 * time.Second)
		go pixmapFileSnapshotWorker(snapshotTicker.C, pixmap)
	}

	waitGroup.Wait()
}

func parseArguments() {
	parser := argparse.NewParser("pixelflut", "a pixel drawing game for programmers (server)")

	argTcpPort = parser.String("t", "tcp", &argparse.Options{
		Help: "Listen for TCP connections on the specified port",
	})
	argWebsocketPort = parser.String("w", "websocket", &argparse.Options{
		Help: "Listen for Websocket connections on the specified port",
	})
	argUdpPort = parser.String("u", "udp", &argparse.Options{
		Help: "Listen fo UDP messages on the specified port",
	})

	argSnapshotFile = parser.File("f", "file", os.O_RDWR|os.O_CREATE, 0640, &argparse.Options{
		Help: "Use this file to periodically save the current canvas into nd load from this file at startup if it contains valid data",
	})
	argNewPixmap = parser.Flag("n", "new", &argparse.Options{
		Help: "Create a new pixmap even if --file parameter is given as well. " +
			"Creating a new pixmap is the default when no --file argument is given.",
	})

	xSizeInt := parser.Int("x", "argPixmapWidth", &argparse.Options{
		Required: false,
		Help:     "Size of the canvas in x dimension",
		Default:  800,
	})
	ySizeInt := parser.Int("y", "argPixmapHeight", &argparse.Options{
		Required: false,
		Help:     "Size of the canvas in y dimension",
		Default:  600,
	})

	if err := parser.Parse(os.Args); err != nil {
		fmt.Println(parser.Usage(err))
		os.Exit(1)
	}

	if *xSizeInt < 0 {
		fmt.Println(parser.Usage("Pixmap Width cannot be smaller than 0"))
		os.Exit(1)
	} else {
		argPixmapWidth = uint(*xSizeInt)
	}

	if *ySizeInt < 0 {
		fmt.Println(parser.Usage("Pixmap Height cannot be smaller than 0"))
		os.Exit(1)
	} else {
		argPixmapHeight = uint(*ySizeInt)
	}
}

func initPixmap() *protocol.Pixmap {
	if *argSnapshotFile != (os.File{}) && !*argNewPixmap {
		if pixmap, err := protocol.NewPixmapFromSnapshot(argSnapshotFile, argPixmapWidth, argPixmapHeight); err != nil {
			fmt.Printf("Could not read pixmap snapshot: %v\n", err)
			fmt.Println("Initializing new pixmap instead")
			return protocol.NewPixmap(argPixmapWidth, argPixmapHeight, []byte{0, 0, 0})
		} else {
			fmt.Printf("Loaded pixmap from sneapshot.\n")
			return pixmap
		}
	} else {
		fmt.Printf("Initialized new pixmap of size %vx%v\n", argPixmapWidth, argPixmapHeight)
		return protocol.NewPixmap(argPixmapWidth, argPixmapHeight, []byte{0, 0, 0})
	}
}

func pixmapStateWorker(tick <-chan time.Time, pixmap *protocol.Pixmap) {
	for {
		<-tick
		pixmap.CalculateStates()
	}
}

func pixmapFileSnapshotWorker(tick <-chan time.Time, pixmap *protocol.Pixmap) {
	for {
		<-tick
		if err := pixmap.WriteToFile(argSnapshotFile); err != nil {
			fmt.Printf("Could not write snapshot to file: %v\n", err)
		}
	}
}
