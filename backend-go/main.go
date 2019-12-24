package main

import (
	"pixelflut.backend/network"
	"pixelflut.backend/protocol"
	"time"
)

func pixmapStateWorker(tick <-chan time.Time, pixmap *protocol.Pixmap) {
	for {
		<-tick
		pixmap.CalculateStateCustomBinary()
	}
}

func main() {
	pixmap := protocol.NewPixmap(800, 600, protocol.Color{})

	stateTicker := time.NewTicker(100 * time.Millisecond)
	go pixmapStateWorker(stateTicker.C, pixmap)

	network.Start("8080", pixmap)
}
