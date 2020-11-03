use crate::i18n::get_catalog;
use crate::net::framing::Frame;
use crate::parser;
use crate::parser::command::*;
use crate::pixmap::SharedPixmap;

mod framing;
pub mod tcp_server;

fn handle_frame(input: Frame, pixmap: &SharedPixmap) -> Option<Frame> {
    // try parse the received frame as command
    let command = match input {
        Frame::Simple(command_str) => match parser::simple::parse(&command_str) {
            Ok((_, command)) => Ok(command),
            Err(_e) => Err("unhelpful, unexplained, generic error"), // TODO improve parser error handling
        },
    };

    // handle the command and construct an appropriate response
    match command {
        Err(e) => Some(Frame::Simple(format!(
            "There was an error parsing your command: {}",
            e
        ))),
        Ok(cmd) => match handle_command(cmd, pixmap) {
            Err(e) => Some(Frame::Simple(format!(
                "There was an error handling your command: {}",
                e
            ))),
            Ok(None) => None,
            Ok(Some(response)) => Some(Frame::Simple(response)),
        },
    }
}

fn handle_command(
    cmd: Command,
    pixmap: &SharedPixmap,
) -> core::result::Result<Option<String>, String> {
    match cmd {
        Command::Size => Ok(Some(format!(
            "SIZE {} {}",
            pixmap.get_size().0,
            pixmap.get_size().1
        ))),
        Command::Help(HelpTopic::General) => Ok(Some(i18n!(get_catalog(), "help_general"))),
        Command::Help(HelpTopic::Size) => Ok(Some(i18n!(get_catalog(), "help_size"))),
        Command::Help(HelpTopic::Px) => Ok(Some(i18n!(get_catalog(), "help_px"))),
        Command::Help(HelpTopic::State) => Ok(Some(i18n!(get_catalog(), "help_state"))),
        Command::PxGet(x, y) => match pixmap.get_pixel(x, y) {
            Some(color) => Ok(Some(format!("PX {} {} {}", x, y, color.to_string()))),
            None => Err("Coordinates are not inside this canvas".to_string()),
        },
        Command::PxSet(x, y, color) => match pixmap.set_pixel(x, y, color) {
            true => Ok(None),
            false => Err("Coordinates are not inside this canvas".to_string()),
        },
    }
}
