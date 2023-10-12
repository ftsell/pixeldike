use crate::net::fixed_msg_stream::FixedMsgStream;
use crate::net::servers::gen_server::{GenServer, GenServerHandle};
use crate::net::MsgReader;
use crate::pixmap::traits::{PixmapRead, PixmapWrite};
use crate::pixmap::SharedPixmap;
use crate::state_encoding::SharedMultiEncodings;
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::net::{TcpStream, UdpSocket};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UdpServerOptions {
    pub bind_addr: SocketAddr,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UdpServer {
    options: UdpServerOptions,
}

impl UdpServer {
    async fn listen<P>(
        pixmap: SharedPixmap<P>,
        encodings: SharedMultiEncodings,
        socket: UdpSocket,
    ) -> anyhow::Result<()>
    where
        P: PixmapRead + PixmapWrite + Send + Sync + 'static,
    {
        loop {
            let mut buffer = FixedMsgStream::<512>::new();
            socket.recv(buffer.get_buf_mut()).await?;
            while crate::net::handle_streams_once(
                &mut buffer,
                Option::<&mut TcpStream>::None,
                &pixmap,
                &encodings,
            )
            .await
            .is_ok()
            {}
        }
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
    ) -> anyhow::Result<GenServerHandle>
    where
        P: PixmapRead + PixmapWrite + Send + Sync + 'static,
    {
        let socket = UdpSocket::bind(self.options.bind_addr).await?;
        tracing::info!("Started UDP Server on {}", self.options.bind_addr);

        let handle = tokio::spawn(async move { UdpServer::listen(pixmap, encodings, socket).await });

        Ok(GenServerHandle::new(handle))
    }
}

struct BufferedUdpReceiver<const BUF_SIZE: usize> {
    socket: UdpSocket,
    buffer: [u8; BUF_SIZE],
    fill_marker: usize,
    msg_marker: usize,
}

#[async_trait]
impl<const BUF_SIZE: usize> MsgReader for BufferedUdpReceiver<BUF_SIZE> {
    async fn read_message(&mut self) -> std::io::Result<&[u8]> {
        todo!()
    }
}
