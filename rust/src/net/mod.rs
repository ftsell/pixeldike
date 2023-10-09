//!
//! Networking layer for pixelflut servers and clients as well as on-the-wire protocol handling
//!

use crate::net_protocol::{Request, Response, StateEncodingAlgorithm};

use nom::Finish;

// use crate::net::framing::{Frame, OldFrame};
use crate::pixmap::traits::{PixmapRead, PixmapWrite};
use crate::pixmap::SharedPixmap;
use crate::state_encoding::SharedMultiEncodings;

mod stream;

pub use stream::{MsgReader, MsgWriter};

#[cfg(feature = "tcp_server")]
pub mod tcp_server;

#[cfg(feature = "udp_server")]
pub mod udp_server;

mod buf_msg_reader;
pub mod tcp_client;
#[cfg(feature = "ws_server")]
pub mod ws_server;

/// Handle one request from the given ReadStream and optionally write a response back into the given WriteStream.
async fn handle_streams_once<P>(
    reader: &mut impl MsgReader,
    writer: Option<&mut impl MsgWriter>,
    pixmap: &SharedPixmap<P>,
    encodings: &SharedMultiEncodings,
) -> anyhow::Result<()>
where
    P: PixmapRead + PixmapWrite,
{
    let msg = reader.read_message().await?;
    let parse_result = crate::net_protocol::parse_request(msg).finish();

    match parse_result {
        Err(_) => {
            match std::str::from_utf8(msg) {
                Ok(msg) => log::info!("received invalid request: {:?}", msg),
                Err(_) => log::info!("received invalid request: {:?}", msg),
            }
            Ok(())
        }
        Ok((_, request)) => match request {
            Request::Help(topic) => handle_response(writer, Response::Help(topic)).await,
            Request::GetSize => {
                let (width, height) = pixmap.get_size()?;
                handle_response(writer, Response::Size { width, height }).await
            }
            Request::GetPixel { x, y } => {
                let color = pixmap.get_pixel(x, y)?;
                handle_response(writer, Response::PxData { x, y, color }).await
            }
            Request::SetPixel { x, y, color } => {
                pixmap.set_pixel(x, y, color)?;
                Ok(())
            }
            Request::GetState(alg) => match alg {
                StateEncodingAlgorithm::Rgb64 => {
                    let state = { encodings.rgb64.lock().unwrap().clone() };
                    handle_response(
                        writer,
                        Response::State {
                            alg,
                            data: state.as_bytes(),
                        },
                    )
                    .await
                }
                StateEncodingAlgorithm::Rgba64 => {
                    let state = { encodings.rgba64.lock().unwrap().clone() };
                    handle_response(
                        writer,
                        Response::State {
                            alg,
                            data: state.as_bytes(),
                        },
                    )
                    .await
                }
            },
        },
    }
}

async fn handle_response(writer: Option<&mut impl MsgWriter>, response: Response<'_>) -> anyhow::Result<()> {
    if let Some(writer) = writer {
        writer.write_response(&response).await?
    }
    Ok(())
}
