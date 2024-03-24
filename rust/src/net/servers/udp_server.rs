use crate::net::servers::gen_server::GenServer;
use crate::pixmap::SharedPixmap;
use crate::DaemonResult;
use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::task::{AbortHandle, JoinSet};

/// Options with which the `UdpServer` is configured
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UdpServerOptions {
    /// The address to which the server binds
    pub bind_addr: SocketAddr,
}

/// A server implementation using UDP to receive pixelflut messages.
///
/// *Note*: This server **never** sends data back.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UdpServer {
    options: UdpServerOptions,
}

impl UdpServer {
    /// Start `n` server processes
    pub async fn start_many(
        self,
        pixmap: SharedPixmap,
        n: usize,
        join_set: &mut JoinSet<DaemonResult>,
    ) -> anyhow::Result<Vec<AbortHandle>> {
        let socket = Arc::new(UdpSocket::bind(self.options.bind_addr).await?);
        tracing::info!(
            "Started UDP Server on {} with {} tasks",
            self.options.bind_addr,
            n
        );
        (0..n)
            .map(|i| {
                let pixmap = pixmap.clone();
                let socket = socket.clone();
                let handle = join_set
                    .build_task()
                    .name(&format!("udp_server{}", i))
                    .spawn(async move { UdpServer::listen(pixmap, socket).await })?;
                Ok(handle)
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }

    #[tracing::instrument(skip_all)]
    async fn listen(pixmap: SharedPixmap, socket: Arc<UdpSocket>) -> anyhow::Result<!> {
        loop {
            // fill a buffer from the network
            let mut req_buf = BytesMut::with_capacity(4 * 1024);
            let (_, sender) = socket.recv_buf_from(&mut req_buf).await?;

            // process received commands in the background
            let pixmap = pixmap.clone();
            let socket = socket.clone();
            tokio::spawn(
                async move { Self::handle_requests(sender, req_buf.freeze(), pixmap, socket).await },
            );
        }
    }

    #[tracing::instrument(skip_all, fields(remote = sender.to_string()))]
    async fn handle_requests(
        sender: SocketAddr,
        mut buf: Bytes,
        pixmap: SharedPixmap,
        socket: Arc<UdpSocket>,
    ) {
        tracing::trace!("Received {}KiB UDP datagram: {:?}", buf.len() / 1024, buf);

        let mut resp_buf = BytesMut::with_capacity(4 * 1024).writer();

        // handle all lines contained in the request buffer
        while let Some((i, _)) = buf.iter().enumerate().find(|(_, &b)| b == b'\n') {
            let line = buf.split_to(i + 1);
            let result = super::handle_request(&line, &pixmap);
            match result {
                Err(e) => {
                    resp_buf.write_fmt(format_args!("{}\n", e)).unwrap();
                }
                Ok(Some(response)) => response.write(&mut resp_buf).unwrap(),
                Ok(None) => {}
            }
        }

        // write accumulated responses back to the sender
        tracing::trace!(
            "Sending back {}KiB response: {:?}",
            resp_buf.get_ref().len() / 1024,
            &resp_buf.get_ref()
        );
        if let Err(e) = socket.send_to(resp_buf.get_ref(), sender).await {
            tracing::error!("Error while writing response to {}: {}", sender, e);
        }
    }
}

#[async_trait]
impl GenServer for UdpServer {
    type Options = UdpServerOptions;

    fn new(options: Self::Options) -> Self {
        Self { options }
    }

    async fn start(
        self,
        pixmap: SharedPixmap,
        join_set: &mut JoinSet<DaemonResult>,
    ) -> anyhow::Result<AbortHandle> {
        let socket = Arc::new(UdpSocket::bind(self.options.bind_addr).await?);
        tracing::info!("Started UDP Server on {}", self.options.bind_addr);

        let handle = join_set
            .build_task()
            .name("udp_server")
            .spawn(async move { UdpServer::listen(pixmap, socket).await })?;
        Ok(handle)
    }
}
