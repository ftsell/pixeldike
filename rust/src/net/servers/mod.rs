mod gen_server;

pub use gen_server::GenServer;

#[cfg(feature = "tcp_server")]
mod tcp_server;
#[cfg(feature = "udp_server")]
mod udp_server;

use crate::net::framing::{BufferFiller, BufferedMsgReader, MsgWriter};
use crate::net::protocol::{parse_request, Request, Response, StateEncodingAlgorithm};
use crate::pixmap::traits::{PixmapRead, PixmapWrite};
use crate::pixmap::SharedPixmap;
use crate::state_encoding::SharedMultiEncodings;
use nom::Finish;

#[cfg(feature = "udp_server")]
pub use udp_server::{UdpServer, UdpServerOptions};

#[cfg(feature = "tcp_server")]
pub use tcp_server::{TcpServer, TcpServerOptions};

/// Handle requests in a loop.
///
/// This is the core loop that is run by all servers.
/// The connection to the server implementation done via the `BufferedMsgReader` and `impl MsgWriter` arguments.
async fn handle_requests<const READ_BUF_SIZE: usize, P, R>(
    mut reader: BufferedMsgReader<READ_BUF_SIZE, R>,
    mut writer: Option<impl MsgWriter>,
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
) -> anyhow::Result<!>
where
    P: PixmapRead + PixmapWrite,
    R: BufferFiller,
{
    loop {
        let msg = reader.read_msg().await?;
        let parse_result = parse_request(msg).finish();

        match parse_result {
            Err(_) => match std::str::from_utf8(msg) {
                Ok(msg) => tracing::info!("received invalid request: {:?}", msg),
                Err(_) => tracing::info!("received invalid request: {:?}", msg),
            },
            Ok((_, request)) => match request {
                Request::Help(topic) => handle_response(&mut writer, Response::Help(topic)).await?,
                Request::GetSize => {
                    let (width, height) = pixmap.get_size()?;
                    handle_response(&mut writer, Response::Size { width, height }).await?
                }
                Request::GetPixel { x, y } => {
                    let color = pixmap.get_pixel(x, y)?;
                    handle_response(&mut writer, Response::PxData { x, y, color }).await?
                }
                Request::SetPixel { x, y, color } => {
                    pixmap.set_pixel(x, y, color)?;
                }
                Request::GetState(alg) => match alg {
                    StateEncodingAlgorithm::Rgb64 => {
                        let state = { encodings.rgb64.lock().unwrap().clone() };
                        handle_response(
                            &mut writer,
                            Response::State {
                                alg,
                                data: state.as_bytes(),
                            },
                        )
                        .await?
                    }
                    StateEncodingAlgorithm::Rgba64 => {
                        let state = { encodings.rgba64.lock().unwrap().clone() };
                        handle_response(
                            &mut writer,
                            Response::State {
                                alg,
                                data: state.as_bytes(),
                            },
                        )
                        .await?
                    }
                },
            },
        }
    }
}

/// Handle a response by writing it into the response writer (if one is present)
async fn handle_response(writer: &mut Option<impl MsgWriter>, response: Response<'_>) -> anyhow::Result<()> {
    if let Some(writer) = writer {
        writer.write_response(&response).await?
    }
    Ok(())
}
