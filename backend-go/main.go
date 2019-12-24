package main

import (
	"fmt"
	"github.com/akamensky/argparse"
	"os"
	"pixelflut/network"
	"pixelflut/protocol"
	"time"
)

var tcpPort *string
var xSize uint
var ySize uint

func main() {
	parseArguments()
	pixmap := protocol.NewPixmap(xSize, ySize, protocol.Color{})
	fmt.Printf("Initialized new pixmap of size %vx%v\n", xSize, ySize)
	
	if *tcpPort != "" {
		network.Start(*tcpPort, pixmap)
	}
	
	stateTicker := time.NewTicker(100 * time.Millisecond)
	go pixmapStateWorker(stateTicker.C, pixmap)
}

func pixmapStateWorker(tick <-chan time.Time, pixmap *protocol.Pixmap) {
	for {
		<-tick
		pixmap.CalculateStateCustomBinary()
	}
}

func parseArguments() {
	parser := argparse.NewParser("pixelflut", "a pixel drawing game for programmers (server)")

	tcpPort = parser.String("t", "tcp", &argparse.Options{
		Help: "Listen for TCP connections on the specified port",
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
