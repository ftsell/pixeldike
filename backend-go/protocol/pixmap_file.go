package protocol

import (
	"errors"
	"fmt"
	"os"
)

func (p *Pixmap) WriteToFile(file *os.File) error {
	p.pixmapLock.RLock()
	defer p.pixmapLock.RUnlock()

	header := []byte{byte(p.xSize), byte(p.ySize)}
	_, err := file.WriteAt(append(header, p.pixmap...), 0)
	_ = file.Sync()
	return err
}

func NewPixmapFromSnapshot(file *os.File, pixmapWidth uint, pixmapHeight uint) (*Pixmap, error) {
	fileLength := int((pixmapWidth * pixmapHeight * ColorByteLength)) + 2
	fileContent := make([]byte, fileLength)

	if n, err := file.Read(fileContent); err != nil {
		return nil, err
	} else if n != fileLength {
		return nil, errors.New(fmt.Sprintf("File length is not as it should be. Expected %v bytes but got %v", fileLength, n))
	} else {
		pixmap := NewPixmap(pixmapWidth, pixmapHeight, []byte{0, 0, 0})
		copy(pixmap.pixmap, fileContent[2:])
		return pixmap, nil
	}
}
