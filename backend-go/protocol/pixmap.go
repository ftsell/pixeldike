package protocol

import (
	"encoding/base64"
	"encoding/hex"
	"errors"
	"strings"
	"sync"
)

const (
	COLOR_BYTE_LENGTH = 3 // RGB channels with 1 byte each 0 - 255 (00 - FF)
)

type Color [COLOR_BYTE_LENGTH]byte

func colorFromHexString(s string) (Color, error) {
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

func colorToHexString(color Color) string {
	return strings.ToUpper(hex.EncodeToString(color[:]))
}

type Pixmap struct {
	pixmap                []Color
	pixmapLock            *sync.RWMutex
	xSize                 uint
	ySize                 uint
	stateCustomBinaryLock *sync.RWMutex
	stateCustomBinary     string
}

func NewPixmap(xSize uint, ySize uint, backgroundColor Color) *Pixmap {
	result := new(Pixmap)

	result.pixmap = make([]Color, xSize*ySize)
	result.pixmapLock = new(sync.RWMutex)
	result.xSize = xSize
	result.ySize = ySize
	result.stateCustomBinaryLock = new(sync.RWMutex)
	result.stateCustomBinary = ""

	for ix := uint(0); ix < xSize; ix++ {
		for iy := uint(0); iy < ySize; iy++ {
			_ = result.SetPixel(ix, iy, backgroundColor)
		}
	}

	return result
}

func (p *Pixmap) SetPixel(x uint, y uint, color Color) error {
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

func (p *Pixmap) GetPixel(x, y uint) (Color, error) {
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

func (p *Pixmap) GetState() string {
	state := ""
	p.stateCustomBinaryLock.RLock()
	state = p.stateCustomBinary
	p.stateCustomBinaryLock.RUnlock()
	return state
}

func (p *Pixmap) CalculateStateCustomBinary() {
	resultBytes := make([]byte, p.xSize * p.ySize * 3)

	p.pixmapLock.RLock()
	for i, iColor := range p.pixmap {
		resultBytes[i * 3] = iColor[0]
		resultBytes[i * 3 + 1] = iColor[1]
		resultBytes[i * 3 + 2] = iColor[2]
	}
	p.pixmapLock.RUnlock()

	p.stateCustomBinaryLock.Lock()
	p.stateCustomBinary = base64.StdEncoding.EncodeToString(resultBytes) + "\n"
	p.stateCustomBinaryLock.Unlock()
}
