package protocol

import (
	"encoding/base64"
	"encoding/hex"
	"errors"
	"fmt"
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
	pixmap          []Color
	pixmapLock      *sync.RWMutex
	xSize           uint
	ySize           uint
	stateLock       *sync.RWMutex
	stateRgbBase64  string
	stateRgbaBase64 string
}

func NewPixmap(xSize uint, ySize uint, backgroundColor Color) *Pixmap {
	result := new(Pixmap)

	result.pixmap = make([]Color, xSize*ySize)
	result.pixmapLock = new(sync.RWMutex)
	result.xSize = xSize
	result.ySize = ySize
	result.stateLock = new(sync.RWMutex)
	result.stateRgbBase64 = ""

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

func (p *Pixmap) GetStateRgbBase64() string {
	p.stateLock.RLock()
	defer p.stateLock.RUnlock()
	return  p.stateRgbBase64
}

func (p *Pixmap) GetStateRgbaBase64() string {
	p.stateLock.RLock();
	defer p.stateLock.RUnlock();
	return p.stateRgbaBase64;
}

func (p *Pixmap) CalculateStates() {
	resultRgbBytes := make([]byte, p.xSize*p.ySize*3)
	resultRgbaBytes := make([]byte, p.xSize*p.ySize*4)

	p.pixmapLock.RLock()
	for i, iColor := range p.pixmap {
		resultRgbBytes[i*3] = iColor[0]
		resultRgbBytes[i*3+1] = iColor[1]
		resultRgbBytes[i*3+2] = iColor[2]

		resultRgbaBytes[i*4] = iColor[0]
		resultRgbaBytes[i*4+1] = iColor[1]
		resultRgbaBytes[i*4+2] = iColor[2]
		resultRgbaBytes[i*4+3] = byte(uint(255))
	}
	p.pixmapLock.RUnlock()

	p.stateLock.Lock()
	p.stateRgbBase64 = fmt.Sprintf("STATE %v %v\n", BINARY_ALG_RGB_BASE64, base64.StdEncoding.EncodeToString(resultRgbBytes))
	p.stateRgbaBase64 = fmt.Sprintf("STATE %v %v\n", BINARY_ALG_RGBA_BASE64, base64.StdEncoding.EncodeToString(resultRgbaBytes))
	p.stateLock.Unlock()
}
