use std::ops::RangeTo;
use std::io::{Write, Error, ErrorKind};

pub mod tcp_server;


pub trait PxServer {

    /// Schedule appropriate handler with tokio
    fn start(self);

    /// Handle an incoming string message with the following steps:
    ///
    ///     - Parse it as a PX command\n
    ///     - Call the correct command handler
    ///
    /// The answer channel should be passed down to the correct command_handler so that it
    /// can directly respond if needed.
    ///
    fn handle_message(&self, msg: &String, answer_channel: &mut Write) -> Result<(), Error> {
        // TODO Make handle_message return the answer and send it from the calling task with tokio
        // Check  if the command is a SIZE command
        if msg.eq(&String::from("SIZE")) {
            return self.cmd_get_size(answer_channel);
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
                return self.cmd_get_px(answer_channel, x.unwrap(), y.unwrap());
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
            return self.cmd_set_px( x.unwrap(), y.unwrap(), color);
        }

        return Err(Error::new(ErrorKind::InvalidInput,
                              "Unknown command"));
    }

    fn cmd_get_size(&self, answer_channel: &mut Write) -> Result<(), Error>;

    fn cmd_get_px(&self, answer_channel: &mut Write, x: usize, y: usize) -> Result<(), Error>;

    fn cmd_set_px(&self, x: usize, y: usize, color: String) -> Result<(), Error>;

}
