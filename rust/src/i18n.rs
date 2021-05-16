pub static HELP_GENERAL: &str =
    "pixelflut - a pixel drawing game for programmers inspred by reddits r/place.\n\
\n\
Available subcommands are:\n\
HELP\t- This help message\n\
SIZE\t- Get the current canvas size\n\
PX\t- Get or set one specific pixel color\n\
STATE\t- Get the whole canvas in a specifically encoded format\n\
\n\
All commands end with a newline character (\\n) and need to be sent as UTF-8 encoded strings.\n\
Responses are also always newline terminated.\n\
\n\
More help is available with 'HELP <subcommand>'";

pub static HELP_SIZE: &str = "Syntax:\t\tSIZE\n\
Response:\tSIZE <width> <height>\n\
\n\
Returns the current canvas size.\n\
This server does not support changing the canvas size at runtime so the result can safely be cached";

pub static HELP_PX: &str = "Syntax:\t\tPX <x> <y> [#<rgb>]\n\
Response:\t[PX <x> <y> #<rgb>]\n\
\n\
Gets or sets the pixel color addressed by the coordinates <x> and <y>.\n\
The mode of operation is determined by the third argument (<rgb>) being present or not.\n\
If it is present, the pixel will be set to that color and no response will be sent.\n\
It it is not present, the current color will be returned.\n\
\n\
<x>\t- X position on the canvas counted from the left side\n\
<y>\t- Y position on the canvas counted from the top\n\
<rgb>\t- HEX encoded rgb color (000000 - FFFFFF)";

pub static HELP_STATE: &str = "Syntax:\t\tSTATE <algorithm>\n\
Response:\tSTATE <algorithm> <data>\n\
\n\
Retrieves the complete canvas in an encoding chosen by <algorithm>.\n\
Currently implemented algorithms are the following.\n\
\n\
rgb64:\n\
Each pixel is encoded into 3 bytes for the color channels red, green and blue.\n\
Those bytes are then simply appended to each other in row-major order.\n\
At the end everything is base64 encoded.\n\
\n\
rgba64:\n\
Each pixel is encoded into 4 bytes for the color channels red, green, blue and alpha whereby alpha is always zero.\n\
These bytes are then simply appended to each other in row-major order.\n\
At the end everything is base64 encoded.";
