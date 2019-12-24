package protocol

import (
	"encoding/hex"
	"errors"
	"strings"
	"sync"
)

const (
	COLOR_BYTE_LENGTH = 3 // RGB channels with 1 byte each 0 - 255 (00 - FF)
)

type Color [COLOR_BYTE_LENGTH]byte

func ColorFromHexString(s string) (Color, error) {
	s = strings.ToLower(s)

	buf := make([]byte, COLOR_BYTE_LENGTH)
	if _, err := hex.Decode(buf, []byte(s)); err == nil {
		result := [COLOR_BYTE_LENGTH]byte{}
		for i := range buf {
			result[i] = buf[i]
		}

		return result, nil
	} else {
		return Color{}, err
	}
}

func ColorToHexString(color Color) string {
	return strings.ToUpper(hex.EncodeToString(color[:]))
}

type Pixmap struct {
	pixmap       []Color
	pixmapLock   *sync.RWMutex
	xSize        uint
	ySize        uint
	snapshotLock *sync.RWMutex
	snapshot     string
}

func NewPixmap(xSize uint, ySize uint, backgroundColor Color) *Pixmap {
	result := new(Pixmap)

	result.pixmap = make([]Color, xSize*ySize)
	result.pixmapLock = new(sync.RWMutex)
	result.xSize = xSize
	result.ySize = ySize
	result.snapshotLock = new(sync.RWMutex)
	result.snapshot = ""

	for ix := uint(0); ix < xSize; ix++ {
		for iy := uint(0); iy < ySize; iy++ {
			_ = result.setPixel(ix, iy, backgroundColor)
		}
	}

	return result
}

func (p *Pixmap) setPixel(x uint, y uint, color Color) error {
	if x >= 0 && x < p.xSize && y >= 0 && y < p.ySize {
		i := y*p.xSize + x

		p.pixmapLock.Lock()
		p.pixmap[i] = color
		p.pixmapLock.Unlock()

		return nil
	} else {
		return errors.New("coordinates are not inside pixmap")
	}
}

func (p *Pixmap) getPixel(x, y uint) (Color, error) {
	if x >= 0 && x < p.xSize && y >= 0 && y < p.ySize {
		i := y*p.xSize + x
		var color Color

		p.pixmapLock.RLock()
		color = p.pixmap[i]
		p.pixmapLock.RUnlock()

		return color, nil
	} else {
		return Color{}, errors.New("coordinates are not inside pixmap")
	}
}
