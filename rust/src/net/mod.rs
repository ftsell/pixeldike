use crate::i18n::get_catalog;
use crate::net::framing::Frame;
use crate::parser;
use crate::parser::command::*;
use crate::pixmap::{Pixmap, SharedPixmap};
use std::future::Future;
use tokio::task::JoinHandle;

mod framing;
pub mod tcp_server;
pub mod udp_server;
pub mod ws_server;

static LOG_TARGET: &str = "pixelflut.net";

pub struct NetOptions {
    pub tcp: Option<tcp_server::TcpOptions>,
    pub udp: Option<udp_server::UdpOptions>,
    pub ws: Option<ws_server::WsOptions>,
}

pub fn start_listeners<P>(pixmap: SharedPixmap<P>, options: NetOptions) -> Vec<JoinHandle<()>>
where
    P: Pixmap + Send + Sync + 'static,
{
    let mut handlers = Vec::new();

    if let Some(tcp_options) = options.tcp {
        handlers.push(start_listener(pixmap.clone(), tcp_options, tcp_server::listen));
    }
    if let Some(udp_options) = options.udp {
        handlers.push(start_listener(pixmap.clone(), udp_options, udp_server::listen));
    }
    if let Some(ws_options) = options.ws {
        handlers.push(start_listener(pixmap.clone(), ws_options, ws_server::listen))
    }

    if handlers.len() == 0 {
        warn!(
            target: LOG_TARGET,
            "No listeners configured. This pixelflut server will not be reachable"
        );
    }

    handlers
}

fn start_listener<
    P: Send + Sync + 'static,
    F: FnOnce(SharedPixmap<P>, O) -> G + Send + 'static,
    G: Future<Output = ()> + Send,
    O: Send + 'static,
>(
    pixmap: SharedPixmap<P>,
    options: O,
    listen_function: F,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        listen_function(pixmap, options).await;
    })
}

fn handle_frame<P>(input: Frame, pixmap: &SharedPixmap<P>) -> Option<Frame>
where
    P: Pixmap,
{
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

fn handle_command<P>(cmd: Command, pixmap: &SharedPixmap<P>) -> Result<Option<String>, String>
where
    P: Pixmap,
{
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
