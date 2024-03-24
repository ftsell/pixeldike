//! Client implementation for different transport protocols

mod gen_client;
#[cfg(feature = "tcp")]
mod tcp_client;
#[cfg(feature = "udp")]
mod udp_client;

pub use gen_client::GenClient;
#[cfg(feature = "tcp")]
pub use tcp_client::TcpClient;
#[cfg(feature = "udp")]
pub use udp_client::UdpClient;
