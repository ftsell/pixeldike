pub static HELP_GENERAL: &str = "HELP GENERAL\n\
pixelflut - a pixel drawing game for programmers inspired by reddits r/place.\n\
\n\
Available subcommands are:\n\
HELP\t- This help message\n\
SIZE\t- Get the current canvas size\n\
PX\t- Get or set one specific pixels color\n\
\n\
More detailed descriptions about these subcommands is available by sending 'HELP <subcommand>'\n\
\n\
All commands end with a newline character (\\n) and need to be sent as ASCII encoded strings.\n\
Responses are also always newline terminated.\n";

pub static HELP_SIZE: &str = "HELP SIZE\n\
Syntax:\t\tSIZE\n\
Response:\tSIZE <width> <height>\n\
\n\
Returns the current canvas size.\n\
This server does not support changing the canvas size at runtime so the result can safely be cached\n";

pub static HELP_PX: &str = "HELP PX\n\
Syntax:\t\tPX <x> <y> [<rgb>]\n\
Response:\t[PX <x> <y> <rgb>]\n\
\n\
Gets or sets the pixel color addressed by the coordinates <x> and <y>.\n\
The mode of operation is determined by the third argument (<rgb>) being present or not.\n\
If it is present, the pixel will be set to that color and no response will be sent.\n\
It it is not present, the current color will be returned.\n\
\n\
<x>\t- X position on the canvas counted from the left side\n\
<y>\t- Y position on the canvas counted from the top\n\
<rgb>\t- HEX encoded rgb color (000000 - FFFFFF)\n";
