//!
//! Server for handling the pixelflut protocol over connectionless UDP datagrams
//!

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use crate::net::buf_msg_reader::BufferedMsgReader;
use crate::net::fixed_msg_stream::FixedMsgStream;
use bytes::{Buf, BytesMut};
use tokio::net::{TcpStream, UdpSocket};
use tokio::select;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use crate::pixmap::traits::{PixmapBase, PixmapRead, PixmapWrite};
use crate::pixmap::SharedPixmap;
use crate::state_encoding::SharedMultiEncodings;

/// Options which can be given to [`listen`] for detailed configuration
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UdpOptions {
    /// On which address the server should listen
    pub listen_address: SocketAddr,
    /// Number of tasks that listen for udp traffic.
    pub tasks: usize,
}

impl Default for UdpOptions {
    fn default() -> Self {
        Self {
            listen_address: SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 1234),
            tasks: num_cpus::get() * 2,
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
pub async fn start_listeners<P>(
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
    options: UdpOptions,
) -> Vec<(JoinHandle<anyhow::Result<()>>, Arc<Notify>)>
where
    P: PixmapBase + PixmapRead + PixmapWrite + Send + Sync + 'static,
{
    let mut results = Vec::new();
    let notify = Arc::new(Notify::new());

    let socket = Arc::new(UdpSocket::bind(options.listen_address).await.unwrap());
    tracing::info!("Started udp listener on {}", socket.local_addr().unwrap());

    for _ in 0..options.tasks {
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        let notify = notify.clone();
        let notify2 = notify.clone();
        let socket = socket.clone();
        let handle = tokio::spawn(async move { listen(pixmap, encodings, notify2, socket).await });
        results.push((handle, notify))
    }

    results
}

/// Listen on the udp port defined through *options* while using the given *pixmap* and *encodings*
/// as backing data storage
pub async fn listen<P>(
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
    notify_stop: Arc<Notify>,
    socket: Arc<UdpSocket>,
) -> anyhow::Result<()>
where
    P: PixmapBase + PixmapRead + PixmapWrite + Send + Sync + 'static,
{
    loop {
        let mut buffer = FixedMsgStream::<512>::new();

        select! {
            _ = socket.recv(buffer.get_buf_mut()) => {
                while super::handle_streams_once(&mut buffer, Option::<&mut TcpStream>::None, &pixmap, &encodings).await.is_ok() {};
            },
            _ = notify_stop.notified() => {
                tracing::info!("Stopping udp server on {}", socket.local_addr().unwrap());
                break
            }
        }
    }

    Ok(())
}
