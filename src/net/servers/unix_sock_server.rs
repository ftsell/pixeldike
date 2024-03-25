use crate::net::servers::GenServer;
use crate::pixmap::SharedPixmap;
use crate::DaemonResult;
use anyhow::anyhow;
use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use std::io::Write;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::task::{AbortHandle, JoinSet};

/// Options with which the `UnixSocketServer` is configured
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnixSocketOptions {
    /// The path at which a socket should be created
    pub path: PathBuf,
}

/// A server implementation using unix domain sockets to transport pixelflut messages.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnixSocketServer {
    options: UnixSocketOptions,
}

impl UnixSocketServer {
    #[tracing::instrument(skip_all)]
    async fn handle_listener(listener: UnixListener, pixmap: SharedPixmap) -> anyhow::Result<!> {
        loop {
            let (stream, _) = listener.accept().await?;
            let pixmap = pixmap.clone();
            tokio::spawn(async move {
                if let Err(e) = UnixSocketServer::handle_connection(stream, pixmap).await {
                    tracing::warn!("Got error while handling unix socket stream: {e}");
                }
            });
        }
    }

    #[tracing::instrument(skip_all)]
    async fn handle_connection(mut stream: UnixStream, pixmap: SharedPixmap) -> anyhow::Result<()> {
        const MAX_LINE_LEN: usize = 32;
        tracing::debug!("Client connected");

        let mut req_buf = BytesMut::with_capacity(16 * 1024);
        let mut resp_buf = BytesMut::with_capacity(2 * 1024).writer();
        loop {
            // fill the line buffer from the socket
            let n = stream.read_buf(&mut req_buf).await?;
            if n == 0 {
                return Err(anyhow!("client stream exhausted"));
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
impl GenServer for UnixSocketServer {
    type Options = UnixSocketOptions;

    fn new(options: Self::Options) -> Self {
        Self { options }
    }

    async fn start(
        self,
        pixmap: SharedPixmap,
        join_set: &mut JoinSet<DaemonResult>,
    ) -> anyhow::Result<AbortHandle> {
        let listener = UnixListener::bind(&self.options.path)?;
        tracing::info!("Started unix listener on {}", self.options.path.display());

        let handle = join_set
            .build_task()
            .name("unix_listener")
            .spawn(async move { UnixSocketServer::handle_listener(listener, pixmap).await })?;
        Ok(handle)
    }
}
