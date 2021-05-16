//!
//! Networking layer for pixelflut servers and clients as well as on-the-wire protocol handling
//!

use crate::net::framing::Frame;
use crate::pixmap::{Pixmap, SharedPixmap};
use crate::protocol::{Request, Response, StateEncodingAlgorithm};
use crate::state_encoding::SharedMultiEncodings;
use anyhow::Result;
use bytes::{Buf, Bytes};
use std::convert::TryFrom;

pub mod framing;
pub mod tcp_server;
pub mod udp_server;
pub mod ws_server;

//static LOG_TARGET: &str = "pixelflut.net";

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
