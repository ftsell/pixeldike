extern crate futures;

use std::io::{Error, ErrorKind};
use std::ops::RangeInclusive;

pub mod tcp_server;
pub mod udp_server;
pub mod websocket_server;

pub trait PxServer {
    /// Schedule appropriate handler with tokio
    fn start(self, port: u16);

    /// Handle an incoming string message with the following steps:
    ///
    ///     - Parse it as a PX command\n
    ///     - Call the correct command handler
    ///
    /// The answer channel should be passed down to the correct command_handler so that it
    /// can directly respond if needed.
    ///
    fn handle_message(&self, msg: &String) -> std::io::Result<Option<String>> {
        let msg = msg.to_uppercase();

        // Check  if the command is a SIZE command
        if msg.eq(&String::from("SIZE")) {
            return self.cmd_get_size().map_err(map_cmd_error_type);
        }
        // Check if it is a PX command
        else if msg.starts_with("PX") {
            // Define iterator over all fields in command and ignore PX part at the beginning
            let mut msg_iterator = msg.split_whitespace();
            msg_iterator.next();

            // Extract values from command
            let x = msg_iterator.next();
            let y = msg_iterator.next();
            let color = msg_iterator.next();

            // Check that necessary parameters could be read
            if x.is_none() || y.is_none() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Could not read X Y parameters",
                ));
            }

            let x = x.unwrap().parse::<usize>();
            let y = y.unwrap().parse::<usize>();

            // Check that X and Y could be parsed
            if x.is_err() || y.is_err() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Could not parse coordinates from X and Y parameters",
                ));
            }

            // If not color is present at all -> GET_PX command
            if color.is_none() {
                return self
                    .cmd_get_px(x.unwrap(), y.unwrap())
                    .map_err(map_cmd_error_type);
            }

            // Extract color parameter with default transparency value
            let color = {
                if color.unwrap().len() == 6 {
                    (color.unwrap().to_string() + "FF")
                } else if color.unwrap().len() == 8 {
                    color.unwrap().to_string()
                } else {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Color parameter has incorrect length",
                    ));
                }
            }
                .to_uppercase();

            // Check that color is valid HEX
            for i in color.chars() {
                if !i.is_ascii_hexdigit() {
                    return Err(Error::new(ErrorKind::InvalidData, "Color is not HEX"));
                }
            }

            // If all checks passed -> Set the pixel
            return self
                .cmd_set_px(x.unwrap(), y.unwrap(), color)
                .map_err(map_cmd_error_type);
        }

        // Check if it is a STATE command
        else if msg.starts_with("STATE") {
            // Define iterator over all fields in command and ignore PX part at the beginning
            let mut msg_iterator = msg.split_whitespace();
            msg_iterator.next();

            // Extract values from command
            let x_start = msg_iterator.next();
            let x_end = msg_iterator.next();
            let y_start = msg_iterator.next();
            let y_end = msg_iterator.next();

            // Check that all parameters could be read
            if x_start.is_none() || x_end.is_none() || y_start.is_none() || y_end.is_none() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Incorrect number of arguments. \
                     Expected X_START X_END Y_START Y_END",
                ));
            }

            // Parse parameters to correct type
            let x_start = x_start.unwrap().parse::<usize>();
            let x_end = x_end.unwrap().parse::<usize>();
            let y_start = y_start.unwrap().parse::<usize>();
            let y_end = y_end.unwrap().parse::<usize>();

            // Check that parsing was successful
            if x_start.is_err() || x_end.is_err() || y_start.is_err() || y_end.is_err() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Arguments could not be parsed as ranges",
                ));
            }

            // Form ranges from parameters
            let x = RangeInclusive::new(x_start.unwrap(), x_end.unwrap());
            let y = RangeInclusive::new(y_start.unwrap(), y_end.unwrap());

            // Check that start and end of ranges are valid

            // If all checks passed -> execute command
            return self.cmd_get_state(x, y).map_err(map_cmd_error_type);
        }

        // Check if it is HELP command
        else if msg.starts_with("HELP") {
            return self.cmd_help(&msg.replace("HELP ", ""))
                .map_err(map_cmd_error_type);
        }

        return Err(Error::new(ErrorKind::InvalidInput, "Unknown command"));
    }

    fn cmd_get_size(&self) -> Result<Option<String>, String>;

    fn cmd_get_px(&self, x: usize, y: usize) -> Result<Option<String>, String>;

    fn cmd_set_px(&self, x: usize, y: usize, color: String) -> Result<Option<String>, String>;

    fn cmd_get_state(
        &self,
        x: RangeInclusive<usize>,
        y: RangeInclusive<usize>,
    ) -> Result<Option<String>, String>;

    fn cmd_help(&self, subcommand: &String) -> Result<Option<String>, String> {
        return if subcommand == "SIZE" {
            Ok(Some(format!("Syntax: 'SIZE'\n\n\
            \
            Retreive the current canvas size in the format 'SIZE $width $height'.\n\
            The current response is '{}'",
                            self.cmd_get_size().unwrap().unwrap().replace("\n", ""))
                .to_string()))
        } else if subcommand == "PX" {
            Ok(Some("Syntax: 'PX $x $x [$rgb | $rgba]'\n\n\
            \
            You can retrieve or set the pixel color at the specified position.\n\
            If no color is specified you will GET an answer containing the color at the specified \
            position.\nIf a color is specified you will SET the color at the specified position\n\n\
            \
            $x : X position on the canvas.\n\
            $y: Y position on the canvas.\n\
            $rgb : color in HEX-encoded rgb format (000000 - FFFFFF)\n\
            $rgba: color in HEX-encoded rgba format (00000000 - FFFFFFFF)".to_string()))
        } else if subcommand == "STATE" {
            Ok(Some("Syntax: 'STATE $x_start $x_end $y_start $y_end'\n\n\
            \
            Receive color state in bulk in a specially encoded format. The bulk is specified \
            using the X and Y ranges specified by $x/y_start and $x/y_end.\n\
            The response is in the format 'STATE $x_start $x_end $y_start $y_end,$color1,$color2,...' \
            where $colorN denotes the color at the nth pixel in HEX-encoded RGBA format.\n\
            The list is ordered by columns and then rows.\n\n\
            \
            $x_start : Start coordinate of width range\n\
            $x_end : End coordinate of width range\n\
            $y_start: Start coordinate of height range\n\
            $y_end: End coordinate of height range".to_string()))
        } else {
            Ok(Some("pixelflut - a pixel drawing game for programmers inspired by \
            reddits r/place.\n\n\
            \
            Available subcommands are:\n\
            HELP - This help message\n\
            SIZE - Get the current canvas size\n\
            PX - Get or set one specific pixel\n\
            STATE - Get multiple pixels in a specific format\n\n\
            \
            All commands end with a newline character (\\n). \
            More help is available with 'HELP $subcommand'.".to_string()))
        };
    }
}

/// Map an error error produced by cmd_*() to an [`Error`]
///
/// [`Error`]: /std/io/struct.Error.html
fn map_cmd_error_type(e: String) -> Error {
    Error::new(ErrorKind::Other, e)
}
