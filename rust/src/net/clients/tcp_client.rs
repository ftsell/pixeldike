use crate::net::clients::GenClient;
use crate::net::protocol::{parse_response_str, Request, Response};
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

/// A pixelflut client that uses TCP and buffered read/write for communication with a pixelflut server.
#[derive(Debug)]
pub struct TcpClient {
    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>,
}

impl TcpClient {
    /// Flush the write buffer to immediately send all enqueued requests to the server.
    async fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush().await
    }

    /// Get the raw writer that is connected to the pixelflut server
    pub fn get_writer(&mut self) -> &mut BufWriter<impl AsyncWrite> {
        &mut self.writer
    }
}

#[async_trait]
impl GenClient for TcpClient {
    async fn connect(addr: SocketAddr) -> std::io::Result<Self> {
        let (reader, writer) = TcpStream::connect(addr).await?.into_split();
        Ok(Self {
            reader: BufReader::new(reader),
            writer: BufWriter::new(writer),
        })
    }

    async fn send_request(&mut self, request: Request) -> std::io::Result<()> {
        request.write_async(&mut self.writer).await
    }

    async fn await_response(&mut self) -> anyhow::Result<Response> {
        let mut buf = String::with_capacity(32);
        self.reader.read_line(&mut buf).await?;
        let response = parse_response_str(&buf)?;
        Ok(response)
    }

    async fn exchange(&mut self, request: Request) -> anyhow::Result<Response> {
        self.send_request(request).await?;
        self.flush().await?;
        let response = self.await_response().await?;
        Ok(response)
    }
}
