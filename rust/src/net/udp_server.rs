//!
//! Server for handling the pixelflut protocol over connectionless UDP datagrams
//!

use crate::net::framing::Frame;
use crate::pixmap::{Pixmap, SharedPixmap};
use crate::state_encoding::SharedMultiEncodings;
use bytes::{Buf, BytesMut};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::UdpSocket;

static LOG_TARGET: &str = "pixelflut.net.udp";

/// Options which can be given to [`listen`] for detailed configuration
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UdpOptions {
    /// On which address the server should listen
    pub listen_address: SocketAddr,
}

impl Default for UdpOptions {
    fn default() -> Self {
        Self {
            listen_address: SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 1234),
        }
    }
}

/// Start the udp server
///
/// This binds to the socket address specified via *options* with UDP.
///
/// It uses the provided *pixmap* as pixel data storage and *encodings* for reading cached state command results.
pub async fn listen<P>(
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
    options: UdpOptions,
) -> tokio::io::Result<()>
where
    P: Pixmap + Send + Sync + 'static,
{
    let socket = Arc::new(UdpSocket::bind(options.listen_address).await?);
    info!(
        target: LOG_TARGET,
        "Started udp listener on {}",
        socket.local_addr().unwrap()
    );

    loop {
        let socket = socket.clone();
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        let mut buffer = BytesMut::with_capacity(1024);
        let (_num_read, origin) = socket.recv_from(&mut buffer[..]).await?;

        tokio::spawn(async move {
            process_received(buffer, origin, socket, pixmap, encodings).await;
        });
    }
}

async fn process_received<P, B>(
    mut buffer: B,
    origin: SocketAddr,
    socket: Arc<UdpSocket>,
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
) where
    P: Pixmap,
    B: Buf + Clone,
{
    // extract frames from received package
    while buffer.has_remaining() {
        match Frame::from_input(buffer.clone()) {
            Err(_) => return,
            Ok((frame, length)) => {
                buffer.advance(length);

                // handle the frame
                match super::handle_frame(frame, &pixmap, &encodings) {
                    None => {}
                    Some(response) => {
                        // send back a response
                        match socket
                            .send_to(&response.encode(), origin) // TODO Find a cleaner way to convert frame to &[u8]
                            .await
                        {
                            Err(e) => {
                                warn!(target: LOG_TARGET, "Error writing frame: {}", e);
                                return;
                            }
                            Ok(_) => {}
                        }
                    }
                }
            }
        }
    }
}
