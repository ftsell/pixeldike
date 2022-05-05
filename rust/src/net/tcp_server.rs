//!
//! Server for handling the pixelflut protocol over TCP connections
//!

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use anyhow::Error;
use bytes::buf::Take;
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use crate::net::framing::Frame;
use crate::pixmap::{Pixmap, SharedPixmap};
use crate::state_encoding::SharedMultiEncodings;

static LOG_TARGET: &str = "pixelflut.net.tcp";

/// Options which can be given to [`listen`] for detailed configuration
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TcpOptions {
    /// On which address the server should listen
    pub listen_address: SocketAddr,
}

impl Default for TcpOptions {
    fn default() -> Self {
        Self {
            listen_address: SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 1234),
        }
    }
}

/// Start the tcp server on a new task.
///
/// This binds to the socket address specified via *options* with TCP and
/// uses the provided *pixmap* as a pixel data storage and *encodings* for reading cached state command results.
///
/// It returns a JoinHandle to the task that is executing the server logic as well as a
/// Notify instance that can be used to stop the server.
pub fn start_listener<P>(
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
    options: TcpOptions,
) -> (JoinHandle<tokio::io::Result<()>>, Arc<Notify>)
where
    P: Pixmap + Send + Sync + 'static,
{
    let notify = Arc::new(Notify::new());
    let notify2 = notify.clone();
    let handle = tokio::spawn(async move { listen(pixmap, encodings, options, notify2).await });

    (handle, notify)
}

/// Listen on the tcp port defined through *options* while using the given *pixmap* and *encodings*
/// as backing data storage
pub async fn listen<P>(
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
    options: TcpOptions,
    notify_stop: Arc<Notify>,
) -> tokio::io::Result<()>
where
    P: Pixmap + Send + Sync + 'static,
{
    let mut connection_stop_notifies = Vec::new();
    let listener = TcpListener::bind(options.listen_address).await?;
    info!(
        target: LOG_TARGET,
        "Started tcp server on {}",
        listener.local_addr().unwrap()
    );

    loop {
        select! {
            res = listener.accept() => {
                let (socket, _) = res?;
                let pixmap = pixmap.clone();
                let encodings = encodings.clone();
                let connection_stop_notify = Arc::new(Notify::new());
                connection_stop_notifies.push(connection_stop_notify.clone());
                tokio::spawn(async move {
                    process_connection(
                        TcpConnection::new(socket),
                        pixmap,
                        encodings,
                        connection_stop_notify,
                    )
                    .await;
                });
            },
            _ = notify_stop.notified() => {
                log::info!("Stopping tcp server on {}", listener.local_addr().unwrap());
                for i_notify in connection_stop_notifies.iter() {
                    i_notify.notify_one();
                }
                break Ok(());
            }
        };
    }
}

async fn process_connection<P>(
    mut connection: TcpConnection,
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
    notify_stop: Arc<Notify>,
) where
    P: Pixmap,
{
    debug!(
        target: LOG_TARGET,
        "Client connected {}",
        connection.stream.peer_addr().unwrap()
    );
    loop {
        // receive a frame from the client
        select! {
            frame = connection.read_frame() => {
                match frame {
                    Err(e) => {
                        warn!(target: LOG_TARGET, "Error reading frame: {}", e);
                        return;
                    }
                    Ok(frame) => {
                        // handle the frame
                        match super::handle_frame(frame, &pixmap, &encodings) {
                            None => {}
                            Some(response) => {
                                // send back a response
                                match connection.write_frame(response).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        warn!(target: LOG_TARGET, "Error writing frame: {}", e)
                                    }
                                }
                            }
                        }
                    }
                }
            },
            _ = notify_stop.notified() => {
                log::info!("Closing connection to {}", connection.stream.peer_addr().unwrap());
                match connection.stream.shutdown().await {
                    Ok(_) => {}
                    Err(e) => log::warn!("Error closing connection: {}", e)
                }
            }
        }
    }
}

pub(crate) struct TcpConnection {
    stream: TcpStream,
    read_buffer: BytesMut,
}

impl TcpConnection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            read_buffer: BytesMut::with_capacity(256),
            stream,
        }
    }

    pub(self) async fn read_frame(&mut self) -> std::io::Result<Frame<Take<BytesMut>>> {
        loop {
            match Frame::from_input(self.read_buffer.clone()) {
                Ok((frame, length)) => {
                    // discard the frame from the buffer
                    self.read_buffer.advance(length);
                    return Ok(frame);
                }
                Err(_) => {
                    let n = self.stream.read_buf(&mut self.read_buffer).await?;
                    if n == 0 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            Error::msg("eof while reading frame"),
                        ));
                    }
                }
            }
        }
    }

    pub(self) async fn write_frame<B>(&mut self, frame: Frame<B>) -> std::io::Result<()>
    where
        B: Buf,
    {
        self.stream.write_buf(&mut frame.encode()).await?;
        Ok(())
    }
}
