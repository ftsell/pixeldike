package protocol

import (
	"encoding/hex"
	"errors"
	"fmt"
	"strings"
)

const (
	ColorByteLength = 3 // RGB channels with 1 byte each (0 - 255)
)

func colorFromHexString(s string) ([]byte, error) {
	s = strings.ToLower(s)

	if len(s) != ColorByteLength*2 {
		return nil, errors.New(fmt.Sprintf("Hex string is incorrect size. Should be %v but was %v", ColorByteLength*2, len(s)))
	}

	if res, err := hex.DecodeString(s); err != nil {
		return nil, err
	} else {
		return res, nil
	}
}

func colorToHexString(color []byte) (string, error) {
	if len(color) != ColorByteLength {
		return "", errors.New(fmt.Sprintf("color argument is not proper color. A proper color is %v bytes in length, not %v",
			ColorByteLength, len(color)))
	} else {
		return strings.ToUpper(hex.EncodeToString(color[:])), nil
	}
}
