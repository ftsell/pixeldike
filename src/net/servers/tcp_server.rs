use crate::net::servers::GenServer;
use crate::pixmap::SharedPixmap;
use crate::DaemonResult;
use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use std::io::Write;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
                    tracing::warn!("Got error while handling tcp connection: {e}");
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
        const MAX_LINE_LEN: usize = 32;
        tracing::debug!("Client connected");

        let mut req_buf = BytesMut::with_capacity(8 * 1024);
        let mut resp_buf = BytesMut::with_capacity(2 * 1024).writer();
        loop {
            // fill the line buffer from the network
            let n = stream.read_buf(&mut req_buf).await?;
            if n == 0 {
                tracing::debug!("Client stream exhausted, likely disconnected");
                return Ok(());
            }
            tracing::trace!("Received {}KiB stream data: {:?}", n / 1024, req_buf);

            // handle all lines contained in the buffer
            while let Some((i, _)) = req_buf.iter().enumerate().find(|(_, &b)| b == b'\n') {
                let line = req_buf.split_to(i + 1);
                let result = super::handle_request(&line, &pixmap);
                match result {
                    Err(e) => {
                        resp_buf.write_fmt(format_args!("{}\n", e)).unwrap();
                    }
                    Ok(Some(response)) => response.write(&mut resp_buf).unwrap(),
                    Ok(None) => {}
                }
            }

            // clear the buffer if someone is deliberately not sending a newline
            if req_buf.len() > MAX_LINE_LEN {
                tracing::warn!(
                    "Request buffer has {}B but no lines left in it. Client is probably misbehaving.",
                    req_buf.len()
                );
                req_buf.clear();
                resp_buf.write_all("line too long\n".as_bytes()).unwrap();
            }

            // write accumulated responses back to the sender
            if !resp_buf.get_ref().is_empty() {
                tracing::trace!(
                    "Sending back {}KiB response: {:?}",
                    resp_buf.get_ref().len() / 1024,
                    resp_buf.get_ref()
                );
                stream.write_all_buf(resp_buf.get_mut()).await?;
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
