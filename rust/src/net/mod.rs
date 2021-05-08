use crate::i18n::get_catalog;
use crate::net::framing::Frame;
use crate::parser::command::*;
use crate::pixmap::{Pixmap, SharedPixmap};
use crate::state_encoding::SharedMultiEncodings;
use std::future::Future;
use std::str::FromStr;
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

pub fn start_listeners<P>(
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
    options: NetOptions,
) -> Vec<JoinHandle<()>>
where
    P: Pixmap + Send + Sync + 'static,
{
    let mut handlers = Vec::new();

    if let Some(tcp_options) = options.tcp {
        handlers.push(start_listener(
            pixmap.clone(),
            encodings.clone(),
            tcp_options,
            tcp_server::listen,
        ));
    }
    if let Some(udp_options) = options.udp {
        handlers.push(start_listener(
            pixmap.clone(),
            encodings.clone(),
            udp_options,
            udp_server::listen,
        ));
    }
    if let Some(ws_options) = options.ws {
        handlers.push(start_listener(
            pixmap.clone(),
            encodings,
            ws_options,
            ws_server::listen,
        ))
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
    F: FnOnce(SharedPixmap<P>, SharedMultiEncodings, O) -> G + Send + 'static,
    G: Future<Output = ()> + Send,
    O: Send + 'static,
>(
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
    options: O,
    listen_function: F,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        listen_function(pixmap, encodings, options).await;
    })
}

fn handle_frame<P>(input: Frame, pixmap: &SharedPixmap<P>, encodings: &SharedMultiEncodings) -> Option<Frame>
where
    P: Pixmap,
{
    // try parse the received frame as command
    let command = match input {
        Frame::Simple(command_str) => Command::from_str(&command_str),
    };

    // handle the command and construct an appropriate response
    match command {
        Err(e) => Some(Frame::Simple(e.to_string())),
        Ok(cmd) => match handle_command(cmd, pixmap, encodings) {
            Err(e) => Some(Frame::Simple(e.to_string())),
            Ok(None) => None,
            Ok(Some(response)) => Some(Frame::Simple(response)),
        },
    }
}

fn handle_command<P>(
    cmd: Command,
    pixmap: &SharedPixmap<P>,
    encodings: &SharedMultiEncodings,
) -> Result<Option<String>, String>
where
    P: Pixmap,
{
    match cmd {
        Command::Size => Ok(Some(format!(
            "SIZE {} {}",
            pixmap.get_size().unwrap().0,
            pixmap.get_size().unwrap().1
        ))),
        Command::Help(HelpTopic::General) => Ok(Some(i18n!(get_catalog(), "help_general"))),
        Command::Help(HelpTopic::Size) => Ok(Some(i18n!(get_catalog(), "help_size"))),
        Command::Help(HelpTopic::Px) => Ok(Some(i18n!(get_catalog(), "help_px"))),
        Command::Help(HelpTopic::State) => Ok(Some(i18n!(get_catalog(), "help_state"))),
        Command::PxGet(x, y) => match pixmap.get_pixel(x, y) {
            Ok(color) => Ok(Some(format!("PX {} {} {}", x, y, color.to_string()))),
            Err(_) => Err("Coordinates are not inside this canvas".to_string()),
        },
        Command::PxSet(x, y, color) => match pixmap.set_pixel(x, y, color) {
            Ok(_) => Ok(None),
            Err(_) => Err("Coordinates are not inside this canvas".to_string()),
        },
        Command::State(algorithm) => match algorithm {
            StateEncodingAlgorithm::Rgb64 => Ok(Some(format!(
                "STATE rgb64 {}",
                encodings.rgb64.lock().unwrap().clone()
            ))),
            StateEncodingAlgorithm::Rgba64 => Ok(Some(encodings.rgba64.lock().unwrap().clone())),
        },
    }
}
