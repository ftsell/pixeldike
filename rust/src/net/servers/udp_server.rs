use crate::net::framing::{BufferFiller, BufferedMsgReader, MsgWriter, VoidWriter};
use crate::net::servers::gen_server::GenServer;
use crate::pixmap::SharedPixmap;
use crate::DaemonHandle;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

/// Options with which the `UdpServer` is configured
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UdpServerOptions {
    /// The address to which the server binds
    pub bind_addr: SocketAddr,
}

/// A server implementation using UDP to receive pixelflut messages.
///
/// *Note*: This server **never** sends data back.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UdpServer {
    options: UdpServerOptions,
}

impl UdpServer {
    /// Start `n` server processes
    pub async fn start_many(self, pixmap: SharedPixmap, n: usize) -> anyhow::Result<Vec<DaemonHandle>> {
        let socket = Arc::new(UdpSocket::bind(self.options.bind_addr).await?);
        let handles = (0..n)
            .into_iter()
            .map(|_| {
                let pixmap = pixmap.clone();
                let socket = socket.clone();
                let join_handle = tokio::spawn(async move { UdpServer::listen(pixmap, socket).await });
                DaemonHandle::new(join_handle)
            })
            .collect::<Vec<_>>();

        tracing::info!(
            "Started UDP server with {} tasks on {}",
            n,
            self.options.bind_addr
        );
        Ok(handles)
    }

    #[tracing::instrument(skip_all)]
    async fn listen(pixmap: SharedPixmap, socket: Arc<UdpSocket>) -> anyhow::Result<!> {
        const BUF_SIZE: usize = 256;
        let filler = UdpBufferFiller::<BUF_SIZE>::new(socket);
        let reader = BufferedMsgReader::<{ BUF_SIZE * 2 * 2 * 2 }, _>::new_empty(filler);
        super::handle_requests(reader, VoidWriter, pixmap).await
    }
}

#[async_trait]
impl GenServer for UdpServer {
    type Options = UdpServerOptions;

    fn new(options: Self::Options) -> Self {
        Self { options }
    }

    async fn start(self, pixmap: SharedPixmap) -> anyhow::Result<DaemonHandle> {
        let socket = Arc::new(UdpSocket::bind(self.options.bind_addr).await?);
        tracing::info!("Started UDP Server on {}", self.options.bind_addr);

        let handle = tokio::spawn(async move { UdpServer::listen(pixmap, socket).await });

        Ok(DaemonHandle::new(handle))
    }
}

/// A helper struct for implementing `BufferFiller` in a way that only fills completed pixelflut messages into the buffer.
///
/// This works by first receiving data from a udp socket into an intermediate buffer whose size can be configured via
/// the `UdpBufferFiller` generic constant, then checking if the last byte is a \n and then only filling the given
/// buffer if that is the case.
pub(crate) struct UdpBufferFiller<const TMP_BUF_SIZE: usize> {
    socket: Arc<UdpSocket>,
    tmp_buffer: [u8; TMP_BUF_SIZE],
}

impl<const TMP_BUF_SIZE: usize> UdpBufferFiller<TMP_BUF_SIZE> {
    fn new(socket: Arc<UdpSocket>) -> Self {
        Self {
            socket,
            tmp_buffer: [0u8; TMP_BUF_SIZE],
        }
    }
}

#[async_trait]
impl<const TMP_BUF_SIZE: usize> BufferFiller for UdpBufferFiller<TMP_BUF_SIZE> {
    async fn fill_buffer(&mut self, buffer: &mut [u8]) -> anyhow::Result<usize> {
        let bytes_read = self.socket.recv(&mut self.tmp_buffer).await?;

        if bytes_read == 0 {
            tracing::warn!("received invalid empty udp packet");
            return Ok(0);
        }

        if self.tmp_buffer[bytes_read - 1] == '\n' as u8 {
            buffer[..bytes_read].copy_from_slice(&self.tmp_buffer[..bytes_read]);
            Ok(bytes_read)
        } else {
            tracing::warn!("received invalid udp packet without \\n");
            Ok(0)
        }
    }
}

/// A helper struct for assembling udp packets that contain multiple pixelflut messages
#[derive(Debug)]
pub struct UdpPacketAssembler<const BUF_SIZE: usize> {
    socket: UdpSocket,
    write_buffer: [u8; BUF_SIZE],
    fill_marker: usize,
}

impl<const BUF_SIZE: usize> UdpPacketAssembler<BUF_SIZE> {
    /// Create a new instance that writes to the given socket.
    pub fn new(socket: UdpSocket) -> Self {
        Self {
            write_buffer: [0u8; BUF_SIZE],
            fill_marker: 0,
            socket,
        }
    }

    /// How much free space the internal buffer has before it needs to be flushed to the server.
    pub fn free_space(&self) -> usize {
        self.write_buffer.len() - self.fill_marker
    }
}

#[async_trait]
impl<const BUF_SIZE: usize> MsgWriter for UdpPacketAssembler<BUF_SIZE> {
    async fn write_data(&mut self, msg: &[u8]) -> std::io::Result<()> {
        if self.free_space() < msg.len() {
            tracing::warn!("UdpPacketAssembler is full and cannot add more data to its assembly");
            return Err(std::io::Error::from(std::io::ErrorKind::OutOfMemory));
        }

        self.write_buffer[self.fill_marker..][..msg.len()].copy_from_slice(msg);
        self.fill_marker += msg.len();
        Ok(())
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        // write the whole assembled packet to the socket
        let written = self.socket.send(&self.write_buffer[..self.fill_marker]).await?;
        if written != self.fill_marker {
            tracing::warn!("UDP socket did not send the whole assembled packet, only {} out of {} bytes were written to the socket.", written, self.fill_marker);
        }

        // reset the internal buffer so that there is free space again
        self.fill_marker = 0;

        Ok(())
    }
}
