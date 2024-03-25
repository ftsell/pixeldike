use crate::net::protocol::{Request, Response};
use async_trait::async_trait;

/// A trait to unify the different transport protocol clients
///
/// Most clients are implemented for convenience and not for high-performance.
/// If you care about performance and need high control about buffering and write scheduling,
/// it is recommended to implement your own client.
#[async_trait]
pub trait GenClient: Sized {
    /// The parameter given to `connect()` that specifies where to connect to
    type ConnectionParam;

    /// Create a new client by connecting to a server
    async fn connect(addr: Self::ConnectionParam) -> std::io::Result<Self>;

    /// Send a request to the connected server
    async fn send_request(&mut self, request: Request) -> std::io::Result<()>;

    /// Wait for the next response sent from the connected server
    async fn await_response(&mut self) -> anyhow::Result<Response>;

    /// Send a request and wait for a corresponding response
    async fn exchange(&mut self, request: Request) -> anyhow::Result<Response> {
        self.send_request(request).await?;
        let response = self.await_response().await?;
        Ok(response)
    }
}
