use crate::cli;
use crate::cli::TargetDimension;
use bytes::buf::Writer;
use bytes::{BufMut, BytesMut};
use pixeldike::net::clients::{TcpClient, UdpClient, UnixSocketClient};
use pixeldike::net::protocol::{Request, Response};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use url::Url;

pub enum DynClient {
    Tcp(TcpClient),
    Udp(UdpClient),
    Unix(UnixSocketClient),
}

impl DynClient {
    pub async fn connect(url: &Url) -> std::io::Result<Self> {
        tracing::info!("Connecting to pixelflut server at {}", url);
        match url.scheme() {
            #[cfg(feature = "tcp")]
            "tcp" => {
                let addr = url
                    .socket_addrs(|| Some(1234))
                    .expect("Could not resolve servers address")[0];
                Ok(Self::Tcp(TcpClient::connect(&addr).await?))
            }
            #[cfg(feature = "udp")]
            "udp" => {
                let addr = url
                    .socket_addrs(|| Some(1234))
                    .expect("Could not resolve servers address")[0];
                Ok(Self::Udp(UdpClient::connect(&addr).await?))
            }
            "unix" => {
                let path = PathBuf::from(url.path());
                Ok(Self::Unix(UnixSocketClient::connect(&path).await?))
            }
            scheme => panic!("Unsupported url scheme {}", scheme),
        }
    }

    #[allow(unused)]
    async fn send_request(&mut self, request: Request) -> std::io::Result<()> {
        match self {
            DynClient::Tcp(tcp) => tcp.send_request(request).await,
            DynClient::Udp(udp) => udp.send_request(request).await,
            DynClient::Unix(unix) => unix.send_request(request).await,
        }
    }

    #[allow(unused)]
    async fn await_response(&mut self) -> anyhow::Result<Response> {
        match self {
            DynClient::Tcp(tcp) => tcp.await_response().await,
            DynClient::Udp(udp) => udp.await_response().await,
            DynClient::Unix(unix) => unix.await_response().await,
        }
    }

    async fn exchange(&mut self, request: Request) -> anyhow::Result<Response> {
        match self {
            DynClient::Tcp(tcp) => tcp.exchange(request).await,
            DynClient::Udp(udp) => udp.exchange(request).await,
            DynClient::Unix(unix) => unix.exchange(request).await,
        }
    }

    /// Run a generic client loop that fills its command buffer from the provided function.
    ///
    /// `fill_buf` should be a function that fills the provided buffer with pixelflut commands.
    /// It is given `x_min, x_max, y_min, y_max` as additional arguments so that commands can be generated for the right
    /// dimensions.
    ///
    /// If `requires_buf_refresh` is true, then the command is filled per iteration of the client loop.
    /// Otherwise it is only filled once.
    pub async fn run_loop<F>(mut self, fill_buf: F, opts: &cli::CommonClientOps, requires_buf_refresh: bool)
    where
        F: Fn(&mut Writer<BytesMut>, usize, usize, usize, usize),
    {
        // preparation
        let (canvas_width, canvas_height) = self.get_size().await;
        let (x_min, x_max, y_min, y_max) = self.calc_bounds(canvas_width, canvas_height, opts);
        let mut buf = BytesMut::new().writer();

        tracing::info!("Preparing command buffer");
        fill_buf(&mut buf, x_min, x_max, y_min, y_max);

        // main loop
        tracing::info!("Running client loop");
        loop {
            // send whole buffer to server (using the most performant method available)
            tracing::debug!("Sending prepared commands to server");
            match &mut self {
                DynClient::Tcp(tcp) => tcp
                    .get_writer()
                    .write_all(buf.get_ref())
                    .await
                    .expect("Could not write commands to server"),
                DynClient::Unix(unix) => unix
                    .get_writer()
                    .write_all(buf.get_ref())
                    .await
                    .expect("Could not write commands to server"),
                DynClient::Udp(udp) => udp
                    .send_bulk(buf.get_ref())
                    .await
                    .expect("Could not send commands to server"),
            }

            // abort loop if only one iteration is requested
            if !opts.do_loop {
                break;
            }

            // refresh buffer content if required
            if requires_buf_refresh {
                buf.get_mut().clear();
                fill_buf(&mut buf, x_min, x_max, y_min, y_max);
            }
        }
    }

    /// Get the remote canvas's size
    async fn get_size(&mut self) -> (usize, usize) {
        let Response::Size { width, height } = self
            .exchange(Request::GetSize)
            .await
            .expect("Could not retrieve size from pixelflut server")
        else {
            panic!("Server sent invalid response to size request")
        };
        tracing::info!(
            "Successfully exchanged metadata with pixelflut server (width={}, height={})",
            width,
            height
        );
        (width, height)
    }

    /// Determine effective bounds from cli args as well as remote canvas size
    ///
    /// Returns `(x_min, x_max, y_min, y_max)`
    fn calc_bounds(
        &mut self,
        canvas_width: usize,
        canvas_height: usize,
        opts: &cli::CommonClientOps,
    ) -> (usize, usize, usize, usize) {
        let x_min = if opts.x_offset >= canvas_width {
            panic!(
                "given x-offset {} is outside of servers canvas with width {}",
                opts.x_offset, canvas_width
            )
        } else {
            opts.x_offset
        };
        let y_min = if opts.y_offset >= canvas_height {
            panic!(
                "given y-offset {} is outside of servers canvas with height {}",
                opts.y_offset, canvas_height
            )
        } else {
            opts.y_offset
        };
        let x_max = match opts.width {
            TargetDimension::Fill => canvas_width,
            TargetDimension::Specific(width) => {
                if x_min + width >= canvas_width {
                    panic!(
                        "given width {} combined with x-offset {} is outside of server canvas with width {}",
                        width, x_min, canvas_width
                    );
                } else {
                    x_min + width
                }
            }
        };
        let y_max = match opts.height {
            TargetDimension::Fill => canvas_height,
            TargetDimension::Specific(height) => {
                if y_min + height >= canvas_height {
                    panic!(
                        "given height {} combined with y-offset {} is outside of server canvas with height {}",
                        height, y_min, canvas_height
                    );
                } else {
                    y_min + height
                }
            }
        };

        (x_min, x_max, y_min, y_max)
    }
}
