use crate::net::clients::GenClient;
use crate::net::framing::{BufferedMsgReader, NullFiller};
use crate::net::servers::UdpPacketAssembler;
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
pub struct UdpClient<const WRITE_BUF_SIZE: usize> {
    _options: UdpClientOptions,
    writer: UdpPacketAssembler<WRITE_BUF_SIZE>,
}

#[async_trait]
impl<const WRITE_BUF_SIZE: usize> GenClient<0> for UdpClient<WRITE_BUF_SIZE> {
    type Options = UdpClientOptions;
    type MsgWriter = UdpPacketAssembler<WRITE_BUF_SIZE>;
    type BufferFiller = NullFiller;

    async fn connect(options: Self::Options) -> anyhow::Result<Self> {
        let socket = UdpSocket::bind(SocketAddr::from_str("0.0.0.0:0").unwrap()).await?;
        socket.connect(&options.server_addr).await?;
        tracing::info!("Configured UDP client to send to {}", &options.server_addr);
        Ok(Self {
            writer: UdpPacketAssembler::new(socket),
            _options: options,
        })
    }

    #[inline(always)]
    fn get_msg_writer(&mut self) -> &mut Self::MsgWriter {
        &mut self.writer
    }

    fn get_msg_reader(&mut self) -> &mut BufferedMsgReader<0, Self::BufferFiller> {
        unimplemented!()
    }
}
