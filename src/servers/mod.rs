extern crate futures;

use std::io::{Error, ErrorKind};

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
        // TODO Make handle_message return the answer and send it from the calling task with tokio
        // Check  if the command is a SIZE command
        if msg.eq(&String::from("SIZE")) {
            return self.cmd_get_size()
                .map_err(map_cmd_error_type);
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
                return Err(Error::new(ErrorKind::InvalidData,
                                      "Could not read X Y parameters"));
            }

            let x = x.unwrap().parse::<usize>();
            let y = y.unwrap().parse::<usize>();

            // Check that X and Y could be parsed
            if x.is_err() || y.is_err() {
                return Err(Error::new(ErrorKind::InvalidData,
                                      "Could not parse coordinates from X and Y parameters"));
            }

            // If not color is present at all -> GET_PX command
            if color.is_none() {
                return self.cmd_get_px(x.unwrap(), y.unwrap())
                    .map_err(map_cmd_error_type);
            }

            // Extract color parameter with default transparency value
            let color = {
                if color.unwrap().len() == 6 {
                    (color.unwrap().to_string() + "FF")
                } else if color.unwrap().len() == 8 {
                    color.unwrap().to_string()
                } else {
                    return Err(Error::new(ErrorKind::InvalidData,
                                          "Color parameter has incorrect length"));
                }
            }.to_uppercase();

            // Check that color is valid HEX
            for i in color.chars() {
                if !i.is_ascii_hexdigit() {
                    return Err(Error::new(ErrorKind::InvalidData,
                                          "Color is not HEX"));
                }
            }

            // If all checks passed -> Set the pixel
            return self.cmd_set_px(x.unwrap(), y.unwrap(), color)
                .map_err(map_cmd_error_type);
        }

        return Err(Error::new(ErrorKind::InvalidInput,
                              "Unknown command"));
    }

    fn cmd_get_size(&self) -> Result<Option<String>, String>;

    fn cmd_get_px(&self, x: usize, y: usize) -> Result<Option<String>, String>;

    fn cmd_set_px(&self, x: usize, y: usize, color: String) -> Result<Option<String>, String>;
}


/// Map an error error produced by cmd_*() to an [`Error`]
///
/// [`Error`]: /std/io/struct.Error.html
fn map_cmd_error_type(e: String) -> Error {
    Error::new(ErrorKind::Other,
               e)
}
