package main

import (
	"pixelflut.backend/network"
	"pixelflut.backend/protocol"
)

func main() {
	pixmap := protocol.NewPixmap(800, 600, protocol.Color{})
	network.Start("8080", pixmap)
}
