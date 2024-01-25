//! Server implementations for different transport protocols

mod gen_server;

pub use gen_server::GenServer;

mod tcp_server;
mod udp_server;
mod ws_server;

use crate::net::framing::{BufferFiller, BufferedMsgReader, MsgWriter};
use crate::net::protocol::{parse_request, Request, Response, ServerConfig};
use crate::pixmap::SharedPixmap;
use nom::Finish;

pub use tcp_server::{TcpServer, TcpServerOptions};
pub use udp_server::{UdpServer, UdpServerOptions};
pub use ws_server::{WsServer, WsServerOptions};

pub(crate) const SERVER_CONFIG: ServerConfig = ServerConfig {
    max_udp_packet_size: 512,
};

/// Handle requests in a loop.
///
/// This is the core loop that is run by all servers.
/// The connection to the server implementation done via the `BufferedMsgReader` and `impl MsgWriter` arguments.
async fn handle_requests<const READ_BUF_SIZE: usize, R>(
    mut reader: BufferedMsgReader<READ_BUF_SIZE, R>,
    mut writer: impl MsgWriter,
    pixmap: SharedPixmap,
) -> anyhow::Result<!>
where
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
                Request::Help(topic) => writer.write_response(&Response::Help(topic)).await?,
                Request::GetSize => {
                    let (width, height) = pixmap.get_size();
                    writer.write_response(&Response::Size { width, height }).await?
                }
                Request::GetPixel { x, y } => {
                    let color = pixmap.get_pixel(x, y)?;
                    writer.write_response(&Response::PxData { x, y, color }).await?
                }
                Request::SetPixel { x, y, color } => {
                    pixmap.set_pixel(x, y, color)?;
                }
                Request::GetConfig => {
                    writer
                        .write_response(&Response::ServerConfig(SERVER_CONFIG))
                        .await?
                }
            },
        }
    }
}
