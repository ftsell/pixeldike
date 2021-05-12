use crate::i18n::get_catalog;
use crate::net::framing::Frame;
use crate::pixmap::{Pixmap, SharedPixmap};
use crate::protocol::{HelpTopic, Request, Response, StateEncodingAlgorithm};
use crate::state_encoding::SharedMultiEncodings;
use anyhow::Result;
use bytes::{Buf, Bytes};
use std::convert::TryFrom;
use std::future::Future;
use tokio::task::JoinHandle;

pub mod framing;
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

/// handle a request frame and return a response frame
fn handle_frame<P, B>(
    input: Frame<B>,
    pixmap: &SharedPixmap<P>,
    encodings: &SharedMultiEncodings,
) -> Option<Frame<Bytes>>
where
    P: Pixmap,
    B: Buf,
{
    // try parse the received frame as request
    match Request::try_from(input) {
        Err(e) => Some(Frame::new_from_string(e.to_string())),
        Ok(request) => match handle_request(request, pixmap, encodings) {
            Err(e) => Some(Frame::new_from_string(e.to_string())),
            Ok(response) => response.map(|r| r.into()),
        },
    }
}

/// handle a request and return a response
fn handle_request<P>(
    request: Request,
    pixmap: &SharedPixmap<P>,
    encodings: &SharedMultiEncodings,
) -> Result<Option<Response>>
where
    P: Pixmap,
{
    match request {
        Request::Size => Ok(Some(Response::Size(pixmap.get_size()?.0, pixmap.get_size()?.1))),
        Request::Help(topic) => Ok(Some(Response::Help(topic))),
        Request::PxGet(x, y) => Ok(Some(Response::Px(x, y, pixmap.get_pixel(x, y)?))),
        Request::PxSet(x, y, color) => {
            pixmap.set_pixel(x, y, color)?;
            Ok(None)
        }
        Request::State(algorithm) => match algorithm {
            StateEncodingAlgorithm::Rgb64 => Ok(Some(Response::State(
                algorithm,
                encodings.rgb64.lock().unwrap().clone(),
            ))),
            StateEncodingAlgorithm::Rgba64 => Ok(Some(Response::State(
                algorithm,
                encodings.rgba64.lock().unwrap().clone(),
            ))),
        },
    }
}
