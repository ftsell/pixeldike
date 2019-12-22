package protocol

import (
	"fmt"
	"strconv"
	"strings"
)

const (
	COMMAND_HELP  = "help"
	COMMAND_SIZE  = "size"
	COMMAND_PX    = "px"
	COMMAND_STATE = "state"

	BINARY_ALG_CUSTOM = "custom_binary"
)

const (
	HELP = "pixelflut - a pixel drawing game for programmers inspired by reddits r/place.\n" +
		"\n" +
		"Available subcommands are:\n" +
		"HELP	- This help message\n" +
		"SIZE	- Get the current canvas size\n" +
		"PX		- Get or Set one specific pixel color\n" +
		"STATE	- Get the whole canvas in a specific binary format\n" +
		"\n" +
		"All commands end with a newline character (\\n) and need to be sent as a UTF-8 encoded string (numbers as well).\n" +
		"Responses are always newline terminated as well.\n" +
		"More help ist available with 'HELP $subcommand'\n"

	HELP_SIZE = "Syntax: 'SIZE\\n'\n" +
		"Response: 'SIZE $width $height\\n'\n" +
		"\n" +
		"Returns the current canvas size.\n"

	HELP_PX = "Syntax: 'PX $x $y [$rgb]\\n'\n" +
		"Response: 'PX $x $y $rgb\\n'\n" +
		"\n" +
		"Gets or sets the pixel color addressed by the coordinates $x and $y.\n" +
		"The mode of operation is determined by the third argument ($rgb) begin present or not.\n" +
		"If it is present, the pixel will be set to that color and the new color returned.\n" +
		"If it is not present, the current color will only be returned.\n" +
		"\n" +
		"$x	- X position on the canvas counted from the left side.\n" +
		"$y	- Y position on the canvas counted from the top.\n" +
		"$rgb	- HEX encoded rgb format without # symbol (000000 - FFFFFF).\n"

	HELP_STATE = ""	// TODO
)

func ParseAndHandleInput(input string, pixmap *Pixmap) string {
	input = strings.ToLower(input)
	parts := strings.Split(input, " ")

	switch parts[0] {
	case COMMAND_HELP:
		{
			switch len(parts) {
			case 1:
				return HELP
			case 2:
				switch parts[1] {
				case COMMAND_HELP:
					return HELP
				case COMMAND_PX:
					return HELP_PX
				case COMMAND_SIZE:
					return HELP_SIZE
				case COMMAND_STATE:
					return HELP_STATE
				default:
					return "Unknown subcommand.\n"
				}
			default:
				return fmt.Sprintf("HELP command has invalid number of parameters\n")
			}
		}

	case COMMAND_SIZE:
		return fmt.Sprintf("SIZE %s %s\n", pixmap.xSize, pixmap.ySize)

	case COMMAND_STATE:
		{
			// add compatibility to old Rust implementation
			if len(parts) == 1 {
				parts = append(parts, BINARY_ALG_CUSTOM)
			}

			switch parts[2] {
			case BINARY_ALG_CUSTOM:
				panic("implement custom binary state command")
			default:
				return "Unknown algorithm. Send HELP STATE\\n for information about available ones.\n"
			}
		}

	case COMMAND_PX:
		{
			switch len(parts) {
			case 3, 4:
				if x, err := strconv.ParseUint(parts[1], 10, 32); err == nil {
					if y, err := strconv.ParseUint(parts[2], 10, 32); err == nil {

						if (len(parts)) == 3 {
							panic("implement getting a pixel")
						} else {
							if color, err := ColorFromString(parts[3]); err == nil {
								pixmap.setPixel(uint(x), uint(y), color)
							} else {
								return fmt.Sprintf("Could not parse HEX string: %s\n", err)
							}
						}

					} else {
						return "Argument 2 cannot be interpreted as coordinate.\n"
					}
				} else {
					return "Argument 1 cannot be interpreted as coordinate.\n"
				}

			default:
				return "PX command has invalid number of arguments. Should either be 2 or 3.\n"
			}
		}

	default:
		return "Unknown command. Send HELP\\n for detailed usage information\n"
	}

	fmt.Printf("%v", input)
	panic("this state should never be reachable")
}
