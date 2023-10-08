//!
//! Server for handling the pixelflut protocol over TCP connections
//!

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use crate::net::stream::{ReadStream, WriteStream};
use crate::pixmap::traits::{PixmapBase, PixmapRead, PixmapWrite};
use crate::pixmap::SharedPixmap;
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
    P: PixmapBase + PixmapRead + PixmapWrite + Send + Sync + 'static,
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
    P: PixmapBase + PixmapRead + PixmapWrite + Send + Sync + 'static,
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
                        socket,
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

pub(super) struct TcpReadStream<'stream> {
    reader: ReadHalf<'stream>,
    read_buffer: [u8; 256],
    fill_marker: usize,
    msg_marker: usize,
}

impl<'stream> TcpReadStream<'stream> {
    pub fn new(reader: ReadHalf<'stream>) -> Self {
        Self {
            reader,
            /// A buffer into which bytes are read
            read_buffer: [0u8; 256],
            /// A marker indicating to which index the read_buffer is filled with data
            fill_marker: 0,
            /// A marker indicating to which index in the read_buffer messages have already been handled
            msg_marker: 0,
        }
    }
}

#[async_trait]
impl ReadStream for TcpReadStream<'_> {
    async fn read_message(&mut self) -> std::io::Result<&[u8]> {
        // reset the buffer
        self.read_buffer[..self.fill_marker].rotate_left(self.msg_marker);
        self.fill_marker -= self.msg_marker;
        self.msg_marker = 0;

        loop {
            // if a valid frame separator (\n) is part of the buffer, return the content before that
            if let Some((i, _)) = self.read_buffer[..self.fill_marker]
                .iter()
                .enumerate()
                .find(|(_, &c)| c == '\n' as u8)
            {
                // return everything up until the found \n as the message (while excluding the \n itself)
                self.msg_marker = i + 1;
                return Ok(&self.read_buffer[..i]);
            }

            // abort if the buffer has already been filled completely
            if self.fill_marker == self.read_buffer.len() {
                return Err(std::io::Error::from(std::io::ErrorKind::OutOfMemory));
            }

            // read new bytes into the buffer
            self.fill_marker += self
                .reader
                .read(&mut self.read_buffer[self.fill_marker..])
                .await?;
        }
    }
}

pub(super) struct TcpWriteStream<'stream> {
    writer: WriteHalf<'stream>,
}

#[async_trait]
impl WriteStream for TcpWriteStream<'_> {
    async fn write_data(&mut self, msg: &[u8]) -> std::io::Result<()> {
        self.writer.write(msg).await?;
        Ok(())
    }
}

async fn process_connection<P>(
    mut stream: TcpStream,
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
    notify_stop: Arc<Notify>,
) where
    P: PixmapBase + PixmapRead + PixmapWrite,
{
    let peer_addr = stream.peer_addr().unwrap();
    debug!("Client connected {}", peer_addr);

    let (tcp_reader, tcp_writer) = stream.split();
    let mut read_stream = TcpReadStream::new(tcp_reader);
    let mut write_stream = TcpWriteStream { writer: tcp_writer };

    loop {
        tokio::select! {
            result = super::handle_streams_once(&mut read_stream, Some(&mut write_stream), &pixmap, &encodings) => {
                if let Err(e) = result {
                    log::warn!("Could not handle message streams: {}", e);
                    return;
                }
            },
            _ = notify_stop.notified() => {
                log::info!("closing connection to {}", peer_addr);
                match write_stream.writer.shutdown().await {
                    Ok(_) => {},
                    Err(e) => log::warn!("Error closing connection: {}", e)
                }
            }
        }
    }
}
