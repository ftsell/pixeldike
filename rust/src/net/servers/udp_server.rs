use crate::net::framing::{BufferFiller, BufferedMsgReader, MsgWriter, VoidWriter};
use crate::net::servers::gen_server::GenServer;
use crate::pixmap::traits::{PixmapRead, PixmapWrite};
use crate::pixmap::SharedPixmap;
use crate::state_encoding::SharedMultiEncodings;
use crate::DaemonHandle;
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UdpServerOptions {
    pub bind_addr: SocketAddr,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UdpServer {
    options: UdpServerOptions,
}

impl UdpServer {
    #[tracing::instrument(skip_all)]
    async fn listen<P>(
        pixmap: SharedPixmap<P>,
        encodings: SharedMultiEncodings,
        socket: UdpSocket,
    ) -> anyhow::Result<!>
    where
        P: PixmapRead + PixmapWrite + Send + Sync + 'static,
    {
        let buffer = BufferedMsgReader::<512, _>::new_empty(socket);
        super::handle_requests(buffer, Option::<VoidWriter>::None, pixmap, encodings).await
    }
}

#[async_trait]
impl GenServer for UdpServer {
    type Options = UdpServerOptions;

    fn new(options: Self::Options) -> Self {
        Self { options }
    }

    async fn start<P>(
        self,
        pixmap: SharedPixmap<P>,
        encodings: SharedMultiEncodings,
    ) -> anyhow::Result<DaemonHandle>
    where
        P: PixmapRead + PixmapWrite + Send + Sync + 'static,
    {
        let socket = UdpSocket::bind(self.options.bind_addr).await?;
        tracing::info!("Started UDP Server on {}", self.options.bind_addr);

        let handle = tokio::spawn(async move { UdpServer::listen(pixmap, encodings, socket).await });

        Ok(DaemonHandle::new(handle))
    }
}

#[async_trait]
impl BufferFiller for UdpSocket {
    async fn fill_buffer(&mut self, buffer: &mut [u8]) -> anyhow::Result<usize> {
        self.recv(buffer).await.map_err(|e| e.into())
    }
}

#[async_trait]
impl MsgWriter for UdpSocket {
    async fn write_data(&mut self, msg: &[u8]) -> std::io::Result<()> {
        self.send(msg).await?;
        Ok(())
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        // udp data is packet oriented so every call to send() flushes automatically
        Ok(())
    }
}
