use crate::net::servers::GenServer;
use crate::pixmap::SharedPixmap;
use crate::DaemonResult;
use anyhow::anyhow;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::{AbortHandle, JoinSet};
use tokio_tungstenite::tungstenite::Message;

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
        let mut stream = tokio_tungstenite::accept_async(stream).await?;

        loop {
            let request = stream.next().await;
            let request = match &request {
                None => return Err(anyhow!("stream is closed")),
                Some(Err(e)) => return Err(anyhow!("{}", e)),
                Some(Ok(msg)) => match msg {
                    Message::Text(msg) => msg.as_bytes(),
                    Message::Binary(msg) => &msg,
                    Message::Close(_) => return Err(anyhow!("WebSocket connection was closed")),
                    _ => return Err(anyhow!("Got unexpected websocket message: {msg:?}")),
                },
            };
            let result = super::handle_request(request, &pixmap);
            match result {
                Err(e) => stream.send(Message::Text(e)).await?,
                Ok(Some(response)) => stream.send(Message::Text(format!("{}", response))).await?,
                Ok(None) => {}
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
