//!
//! Server for handling the pixelflut protocol over TCP connections
//!

use std::net::{Ipv4Addr, SocketAddr};

use anyhow::Error;
use bytes::buf::Take;
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

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

/// Start the tcp server
///
/// This binds to the socket address specified via *options* with TCP.
///
/// It uses the provided *pixmap* as a pixel data storage and *encodings* for reading cached state command results.
pub async fn listen<P>(
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
    options: TcpOptions,
) -> tokio::io::Result<()>
where
    P: Pixmap + Send + Sync + 'static,
{
    let listener = TcpListener::bind(options.listen_address).await?;
    info!(
        target: LOG_TARGET,
        "Started tcp server on {}",
        listener.local_addr().unwrap()
    );

    loop {
        let (socket, _) = listener.accept().await?;
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        tokio::spawn(async move {
            process_connection(TcpConnection::new(socket), pixmap, encodings).await;
        });
    }
}

async fn process_connection<P>(
    mut connection: TcpConnection,
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
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
        match connection.read_frame().await {
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
