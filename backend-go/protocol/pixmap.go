package protocol

import (
	"encoding/base64"
	"errors"
	"fmt"
	"sync"
)

type Pixmap struct {
	pixmap          []byte
	pixmapLock      *sync.RWMutex
	xSize           uint
	ySize           uint
	stateLock       *sync.RWMutex
	stateRgbBase64  string
	stateRgbaBase64 string
}

func NewPixmap(xSize uint, ySize uint, backgroundColor []byte) *Pixmap {
	result := new(Pixmap)

	result.pixmap = make([]byte, xSize*ySize*ColorByteLength)
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

func (p *Pixmap) SetPixel(x uint, y uint, color []byte) error {
	if x >= 0 && x < p.xSize && y >= 0 && y < p.ySize {
		i := (y*p.xSize + x) * ColorByteLength

		p.pixmapLock.Lock()
		copy(p.pixmap[i:i+3], color)
		p.pixmapLock.Unlock()

		return nil
	} else {
		return errors.New("coordinates are not inside pixmap")
	}
}

func (p *Pixmap) GetPixel(x, y uint) ([]byte, error) {
	if x >= 0 && x < p.xSize && y >= 0 && y < p.ySize {
		i := (y*p.xSize + x) * ColorByteLength
		color := make([]byte, 3)

		p.pixmapLock.RLock()
		copy(color, p.pixmap[i:i+3])
		p.pixmapLock.RUnlock()

		return color, nil
	} else {
		return nil, errors.New("coordinates are not inside pixmap")
	}
}

func (p *Pixmap) GetStateRgbBase64() string {
	p.stateLock.RLock()
	defer p.stateLock.RUnlock()
	return p.stateRgbBase64
}

func (p *Pixmap) GetStateRgbaBase64() string {
	p.stateLock.RLock()
	defer p.stateLock.RUnlock()
	return p.stateRgbaBase64
}

func (p *Pixmap) CalculateStates() {
	resultRgbBytes := make([]byte, p.xSize*p.ySize*3)
	resultRgbaBytes := make([]byte, p.xSize*p.ySize*4)

	{
		p.pixmapLock.RLock()
		defer p.pixmapLock.RUnlock()

		// we already store the pixmap in RGB which means that encoding is a simple copy
		copy(resultRgbBytes, p.pixmap)
		// for RGBA encoding we need to insert one alpha channel for every three bytes
		for i := uint(0); i < p.xSize * p.ySize; i += 1 {
			srcI := i * 3
			destI := i * 4
			copy(resultRgbaBytes[destI:destI+3], p.pixmap[srcI:srcI+3])
			resultRgbaBytes[destI+3] = byte(uint(255))
		}
	}

	p.stateLock.Lock()
	defer p.stateLock.Unlock()
	p.stateRgbBase64 = fmt.Sprintf("STATE %v %v\n", BINARY_ALG_RGB_BASE64, base64.StdEncoding.EncodeToString(resultRgbBytes))
	p.stateRgbaBase64 = fmt.Sprintf("STATE %v %v\n", BINARY_ALG_RGBA_BASE64, base64.StdEncoding.EncodeToString(resultRgbaBytes))
}
