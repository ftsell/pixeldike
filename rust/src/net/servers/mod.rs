//! Server implementations for different transport protocols

mod gen_server;

pub use gen_server::GenServer;

mod tcp_server;
mod udp_server;
mod ws_server;

use crate::net::protocol::{parse_request_bin, Request, Response};
use crate::pixmap::SharedPixmap;

pub use tcp_server::{TcpServer, TcpServerOptions};
pub use udp_server::{UdpServer, UdpServerOptions};
pub use ws_server::{WsServer, WsServerOptions};

/// Handle a single request
///
/// This is the core request handling method that is run by all servers.
/// It parses requests, handles them and generates responses.
/// The actual IO is left to the specific server though.
fn handle_request(line: &[u8], pixmap: &SharedPixmap) -> Result<Option<Response>, String> {
    let parse_result = parse_request_bin(line);
    match parse_result {
        Err(e) => Err(e.to_string()),
        Ok(request) => match request {
            Request::Help(topic) => Ok(Some(Response::Help(topic))),
            Request::GetSize => {
                let (width, height) = pixmap.get_size();
                Ok(Some(Response::Size { width, height }))
            }
            Request::GetPixel { x, y } => {
                let color = pixmap.get_pixel(x, y).map_err(|e| format!("{:?}", e))?;
                Ok(Some(Response::PxData { x, y, color }))
            }
            Request::SetPixel { x, y, color } => {
                pixmap.set_pixel(x, y, color).map_err(|e| format!("{:?}", e))?;
                Ok(None)
            }
        },
    }
}
