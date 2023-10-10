use crate::net::buf_msg_reader::BufferedMsgReader;
use crate::net::msg_streams::{MsgReader, MsgWriter};
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

/// A TCP client that connects to a pixelflut server and can send and receive messages
pub struct TcpClient {
    msg_reader: BufferedMsgReader<OwnedReadHalf>,
    msg_writer: OwnedWriteHalf,
}

impl TcpClient {
    /// Connect to a pixelflut server over tcp
    pub async fn connect(addr: &SocketAddr) -> std::io::Result<Self> {
        let tcp_stream = TcpStream::connect(addr).await?;
        let (reader, writer) = tcp_stream.into_split();

        Ok(Self {
            msg_reader: BufferedMsgReader::<_, 256>::new(reader),
            msg_writer: writer,
        })
    }

    pub fn reader(&mut self) -> &mut impl MsgReader {
        &mut self.msg_reader
    }

    pub fn writer(&mut self) -> &mut impl MsgWriter {
        &mut self.msg_writer
    }

    pub async fn flush(&mut self) -> std::io::Result<()> {
        self.msg_writer.flush().await
    }
}
