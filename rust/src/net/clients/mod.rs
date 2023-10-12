//! Various pixelflut client implementations over different transport protocols

mod gen_client;

#[cfg(feature = "udp_client")]
mod udp_client;
#[cfg(feature = "udp_client")]
pub use udp_client::{UdpClient, UdpClientOptions};

pub use gen_client::GenClient;
