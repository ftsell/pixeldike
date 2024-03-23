use crate::net::framing::{BufferFiller, BufferedMsgReader, MsgWriter};
use crate::net::servers::GenServer;
use crate::pixmap::SharedPixmap;
use crate::DaemonResult;
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::{AbortHandle, JoinSet};

/// Options with which the `TcpServer` is configured
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TcpServerOptions {
    /// The address to which the server binds
    pub bind_addr: SocketAddr,
}

/// A server implementation using TCP to transport pixelflut messages.
#[derive(Debug, Copy, Clone)]
pub struct TcpServer {
    options: TcpServerOptions,
}

impl TcpServer {
    #[tracing::instrument(skip_all)]
    async fn handle_listener(listener: TcpListener, pixmap: SharedPixmap) -> anyhow::Result<!> {
        loop {
            let (stream, remote_addr) = listener.accept().await?;
            let pixmap = pixmap.clone();
            tokio::spawn(async move {
                if let Err(e) = TcpServer::handle_connection(stream, remote_addr, pixmap).await {
                    tracing::error!("Got error while handling tcp connection: {e}");
                }
            });
        }
    }

    #[tracing::instrument(skip_all, fields(remote = _remote_addr.to_string()))]
    async fn handle_connection(
        mut stream: TcpStream,
        _remote_addr: SocketAddr,
        pixmap: SharedPixmap,
    ) -> anyhow::Result<()> {
        tracing::debug!("Client connected");
        let (read_stream, write_stream) = stream.split();
        let buffer = BufferedMsgReader::<512, _>::new_empty(read_stream);
        match super::handle_requests(buffer, write_stream, pixmap).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // handle known errors which are expected and known to be okay
                if let Some(e) = e.downcast_ref::<std::io::Error>() {
                    if let std::io::ErrorKind::UnexpectedEof | std::io::ErrorKind::ConnectionReset = e.kind()
                    {
                        tracing::debug!("Client disconnected");
                        return Ok(());
                    }
                }

                // handle unknown errors by logging and returning them
                tracing::debug!(
                    error = e.to_string(),
                    "Got unexpected error while handling client sinks"
                );
                return Err(e);
            }
        }
    }
}

#[async_trait]
impl GenServer for TcpServer {
    type Options = TcpServerOptions;

    fn new(options: Self::Options) -> Self {
        Self { options }
    }

    async fn start(
        self,
        pixmap: SharedPixmap,
        join_set: &mut JoinSet<DaemonResult>,
    ) -> anyhow::Result<AbortHandle> {
        let listener = TcpListener::bind(self.options.bind_addr).await?;
        tracing::info!("Started TCP Server on {}", self.options.bind_addr);

        let handle = join_set
            .build_task()
            .name("tcp_server")
            .spawn(async move { TcpServer::handle_listener(listener, pixmap).await })?;
        Ok(handle)
    }
}

#[async_trait]
impl BufferFiller for ReadHalf<'_> {
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

#[async_trait]
impl MsgWriter for WriteHalf<'_> {
    async fn write_data(&mut self, msg: &[u8]) -> std::io::Result<()> {
        <Self as AsyncWriteExt>::write(self, msg).await?;
        Ok(())
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        <Self as AsyncWriteExt>::flush(self).await
    }
}
