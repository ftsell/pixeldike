//!
//! Server for handling the pixelflut protocol over connectionless UDP datagrams
//!

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use bytes::{Buf, BytesMut};
use tokio::net::UdpSocket;
use tokio::select;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use crate::net::framing::Frame;
use crate::pixmap::traits::{PixmapBase, PixmapRead, PixmapWrite};
use crate::pixmap::SharedPixmap;
use crate::state_encoding::SharedMultiEncodings;

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

/// Start the udp server on a new task
///
/// This binds to the socket address specified via *options* with UDP and
/// uses the provided *pixmap* as pixel data storage and *encodings* for reading cached state command results.
///
/// It returns a JoinHandle to the task that is executing the server logic as well as a Notify
/// instance that can be used to stop the server.
pub fn start_listener<P>(
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
    options: UdpOptions,
) -> (JoinHandle<tokio::io::Result<()>>, Arc<Notify>)
where
    P: PixmapBase + PixmapRead + PixmapWrite + Send + Sync + 'static,
{
    let notify = Arc::new(Notify::new());
    let notify2 = notify.clone();
    let handle = tokio::spawn(async move { listen(pixmap, encodings, options, notify2).await });

    (handle, notify)
}

/// Listen on the udp port defined through *options* while using the given *pixmap* and *encodings*
/// as backing data storage
pub async fn listen<P>(
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
    options: UdpOptions,
    notify_stop: Arc<Notify>,
) -> tokio::io::Result<()>
where
    P: PixmapBase + PixmapRead + PixmapWrite + Send + Sync + 'static,
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

        select! {
            res = socket.recv_from(&mut buffer[..]) => {
                let (_num_read, origin) = res?;
                tokio::spawn(async move {
                    process_received(buffer, origin, socket, pixmap, encodings).await;
                });
            },
            _ = notify_stop.notified() => {
                log::info!("Stopping udp server on {}", socket.local_addr().unwrap());
                break Ok(());
            }
        }
    }
}

async fn process_received<P, B>(
    mut buffer: B,
    origin: SocketAddr,
    socket: Arc<UdpSocket>,
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
) where
    P: PixmapBase + PixmapRead + PixmapWrite,
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
