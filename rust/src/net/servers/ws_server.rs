use crate::net::framing::{BufferFiller, BufferedMsgReader, MsgWriter};
use crate::net::servers::GenServer;
use crate::pixmap::SharedPixmap;
use crate::DaemonResult;
use anyhow::anyhow;
use async_trait::async_trait;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::{AbortHandle, JoinSet};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

/// Options with which the `WsServer` is configured
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WsServerOptions {
    /// The address to which the server binds
    pub bind_addr: SocketAddr,
}

/// A server implementation using WebSocket to transport pixelflut messages
#[derive(Debug, Copy, Clone)]
pub struct WsServer {
    options: WsServerOptions,
}

impl WsServer {
    #[tracing::instrument(skip_all)]
    async fn handle_listener(listener: TcpListener, pixmap: SharedPixmap) -> anyhow::Result<!> {
        loop {
            let (stream, remote_addr) = listener.accept().await?;
            let pixmap = pixmap.clone();
            tokio::spawn(async move {
                if let Err(e) = WsServer::handle_connection(stream, remote_addr, pixmap).await {
                    tracing::error!("Got error while handling WebSocket connection: {e}");
                }
            });
        }
    }

    #[tracing::instrument(skip_all, fields(remote = _remote_addr.to_string()))]
    async fn handle_connection(
        stream: TcpStream,
        _remote_addr: SocketAddr,
        pixmap: SharedPixmap,
    ) -> anyhow::Result<()> {
        tracing::debug!("Client connected; performing WebSocket handshake");
        let stream = tokio_tungstenite::accept_async(stream).await?;
        let (write_stream, read_stream) = stream.split();
        let buffer = BufferedMsgReader::<512, _>::new_empty(read_stream);

        let result = super::handle_requests(buffer, write_stream, pixmap).await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::debug!(
                    error = e.to_string(),
                    "Got unexpected error while handling client sinks"
                );
                Err(e)
            }
        }
    }
}

#[async_trait]
impl GenServer for WsServer {
    type Options = WsServerOptions;

    fn new(options: Self::Options) -> Self {
        Self { options }
    }

    async fn start(
        self,
        pixmap: SharedPixmap,
        join_set: &mut JoinSet<DaemonResult>,
    ) -> anyhow::Result<AbortHandle> {
        let listener = TcpListener::bind(self.options.bind_addr).await?;
        tracing::info!("Started WebSocket Server on {}", self.options.bind_addr);

        let handle = join_set
            .build_task()
            .name("ws_server")
            .spawn(async move { WsServer::handle_listener(listener, pixmap).await })?;
        Ok(handle)
    }
}

#[async_trait]
impl BufferFiller for SplitStream<WebSocketStream<TcpStream>> {
    async fn fill_buffer(&mut self, buffer: &mut [u8]) -> anyhow::Result<usize> {
        match self.next().await {
            None => Err(anyhow!("stream is closed")),
            Some(Err(e)) => Err(anyhow!(e)),
            Some(Ok(msg)) => match msg {
                Message::Text(msg) => {
                    let msg_bytes = msg.as_bytes();
                    buffer[0..msg_bytes.len()].copy_from_slice(msg_bytes);

                    // be nice and add a \n when the client has not sent it
                    if msg_bytes[msg_bytes.len() - 1] != b'\n' {
                        buffer[msg_bytes.len()] = b'\n';
                        Ok(msg_bytes.len() + 1)
                    } else {
                        Ok(msg_bytes.len())
                    }
                }
                Message::Binary(msg) => {
                    buffer.copy_from_slice(&msg);
                    Ok(msg.len())
                }
                Message::Close(_) => Err(anyhow!("WebSocket connection was closed")),
                _ => Err(anyhow!("Got unexpected websocket message: {msg:?}")),
            },
        }
    }
}

#[async_trait]
impl MsgWriter for SplitSink<WebSocketStream<TcpStream>, Message> {
    async fn write_data(&mut self, msg: &[u8]) -> std::io::Result<()> {
        match self
            .send(Message::Text(String::from_utf8(Vec::from(msg)).unwrap()))
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::debug!("Received unexpected error when sending WebSocket message: {e:?}");
                Err(std::io::ErrorKind::Other.into())
            }
        }
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        match <Self as SinkExt<_>>::flush(self).await {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::debug!("Received unexpected error when flushing WebSocket stream: {e:?}");
                Err(std::io::ErrorKind::Other.into())
            }
        }
    }

    async fn write_message_delimiter(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
