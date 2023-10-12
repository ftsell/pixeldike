use crate::net::clients::GenClient;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::net::UdpSocket;

/// Options with which a `UdpClient` is configured
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UdpClientOptions {
    /// The address of the server to connect to
    pub server_addr: SocketAddr,
}

/// A client that interacts with a pixelflut server over the UDP transport protocol
#[derive(Debug)]
pub struct UdpClient {
    options: UdpClientOptions,
    socket: UdpSocket,
}

#[async_trait]
impl GenClient for UdpClient {
    type Options = UdpClientOptions;
    type MsgWriter = UdpSocket;

    async fn connect(options: Self::Options) -> anyhow::Result<Self> {
        let socket = UdpSocket::bind(SocketAddr::from_str("0.0.0.0:0").unwrap()).await?;
        socket.connect(&options.server_addr).await?;
        tracing::info!("Configured UDP client to send to {}", &options.server_addr);
        Ok(Self { options, socket })
    }

    #[inline(always)]
    fn get_msg_writer(&mut self) -> &mut Self::MsgWriter {
        &mut self.socket
    }
}
