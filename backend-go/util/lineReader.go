package util

import (
	"bytes"
	"io"
)

func ReadLine(reader io.Reader) (string, error) {
	largeBuffer := make([]byte, 0, 512)
	smallBuffer := make([]byte, 64)
	for {
		if _, err := reader.Read(smallBuffer); err != nil {
			return "", err
		} else {
			if i := bytes.IndexRune(smallBuffer, '\n'); i == -1 {
				largeBuffer = append(largeBuffer, smallBuffer...)
			} else {
				return string(append(largeBuffer, smallBuffer[:i]...)), nil
			}
		}
	}
}
