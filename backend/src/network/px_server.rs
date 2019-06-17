use crate::network::protocol::Command;
use crate::color::Color;

pub trait PxServer {
    ///
    /// Start the listener and handle new connections
    ///
    fn start(self, listen_address: &String, port: u16);

    ///
    /// Handle a command received from an arbitrary source and return its result
    ///
    fn handle_command(&self, command: Command) -> Result<String, String> {
        match command {
            Command::Help() => Ok(self.get_help()),
            Command::HelpSubcommand(subcommand) => self.get_help_subcommand(&subcommand),
            Command::Size() => Ok(self.get_size()),
            Command::GetPx(x, y) => self.get_px(x, y),
            Command::SetPx(x, y, color) => self.set_px(x, y, color)
        }
    }

    fn get_help(&self) -> String {
        "pixelflut - a pixel drawing game for programmers inspired by \
         reddits r/place.\n\n\
         \
         Available subcommands are:\n\
         HELP    - This help message\n\
         SIZE    - Get the current canvas size\n\
         PX      - Get or set one specific pixel\n\
         STATE   - Get multiple pixels in a specific format\n\n\
         \
         All commands end with a newline character (\\n) \
         and need to be sent as a string (including numbers)\n\
         More help is available with 'HELP $subcommand'.\n"
            .to_string()
    }

    fn get_help_subcommand(&self, subcommand: &String) -> Result<String, String> {
        match subcommand.as_str() {
            "help" => Ok(self.get_help()),

            "size" => Ok(format!(
                "Syntax: 'SIZE'\n\n\
                 \
                 Retreive the current canvas size in the format 'SIZE $width $height'.\n\
                 The current response would be '{}'\n",
                self.get_size().replace("\n", "")
            )),

            "px" => Ok("Syntax: 'PX $x $x [$rgb | $rgba]'\n\n\
            \
            You can retrieve or set the pixel color at the specified position.\n\
            If no color is specified you will GET a response containing the color at the specified \
            position.\n\
            If a color is specified you will SET the color at the specified position\n\n\
            \
            $x : X position on the canvas.\n\
            $y: Y position on the canvas.\n\
            $rgb : color in HEX-encoded rgb format (000000 - FFFFFF)\n\
            $rgba: color in HEX-encoded rgba format (00000000 - FFFFFFFF)\n".to_string()),

            _ => Err(format!("Unknown subcommand {}\n\n{}", subcommand, self.get_help()))
        }
    }

    fn get_size(&self) -> String;

    fn get_px(&self, x: usize, y: usize) -> Result<String, String>;

    fn set_px(&self, x: usize, y: usize, color: Color) -> Result<String, String>;
}
