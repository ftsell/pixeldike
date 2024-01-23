use crate::net::clients::GenClient;
use crate::net::framing::{BufferFiller, BufferedMsgReader, MsgWriter};
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

/// Options with which a `TcpClient` is configured
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TcpClientOptions {
    /// The address of the server to connect to
    pub server_addr: SocketAddr,
}

/// A client that interacts with a pixelflut server over the TCP transport protocol
#[derive(Debug)]
pub struct TcpClient<const READ_BUF_SIZE: usize> {
    _options: TcpClientOptions,
    reader: BufferedMsgReader<READ_BUF_SIZE, OwnedReadHalf>,
    writer: OwnedWriteHalf,
}

#[async_trait]
impl<const READ_BUF_SIZE: usize> GenClient<READ_BUF_SIZE> for TcpClient<READ_BUF_SIZE> {
    type Options = TcpClientOptions;
    type MsgWriter = OwnedWriteHalf;
    type BufferFiller = OwnedReadHalf;

    async fn connect(options: Self::Options) -> anyhow::Result<Self> {
        let (reader, writer) = TcpStream::connect(options.server_addr).await?.into_split();

        Ok(Self {
            _options: options,
            writer,
            reader: BufferedMsgReader::new_empty(reader),
        })
    }

    fn get_msg_writer(&mut self) -> &mut Self::MsgWriter {
        &mut self.writer
    }

    fn get_msg_reader(&mut self) -> &mut BufferedMsgReader<READ_BUF_SIZE, Self::BufferFiller> {
        &mut self.reader
    }
}

#[async_trait]
impl MsgWriter for OwnedWriteHalf {
    async fn write_data(&mut self, msg: &[u8]) -> std::io::Result<()> {
        <Self as AsyncWriteExt>::write(self, msg).await?;
        Ok(())
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        <Self as AsyncWriteExt>::flush(self).await
    }
}

#[async_trait]
impl BufferFiller for OwnedReadHalf {
    async fn fill_buffer(&mut self, buffer: &mut [u8]) -> anyhow::Result<usize> {
        assert!(buffer.len() > 0);
        match self.read(buffer).await {
            Ok(n) => match n {
                0 => Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into()),
                n => Ok(n),
            },
            Err(e) => Err(e.into()),
        }
    }
}
