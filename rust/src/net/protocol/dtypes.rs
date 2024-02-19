//! Data types that describe all protocol interactions as safe-to-use structs

use crate::pixmap::Color;
use std::fmt::{Display, Formatter};

/// The help topics that can be requested from the server
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HelpTopic {
    /// Help about the general pixelflut protocol and links to further topics
    General,
    /// Help about the *SIZE* command
    Size,
    /// Help about the *PX* command (both set and get variants)
    Px,
}

/// A request to a pixelflut server
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Request {
    /// Request help about a specific topic
    Help(HelpTopic),
    /// Request server configuration i.e. buffer sizes
    GetConfig,
    /// Get the size of the canvas
    GetSize,
    /// Get the color of one pixel from the server
    GetPixel {
        /// The x coordinate of the pixel
        x: usize,
        /// The y coordinate of the pixel
        y: usize,
    },
    /// Set the color of one pixel
    SetPixel {
        /// The x coordinate of the pixel
        x: usize,
        /// The y coordinate of the pixel
        y: usize,
        /// The color to which the pixel should be set
        color: Color,
    },
}

impl Display for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Request::Help(topic) => match topic {
                HelpTopic::General => f.write_str("HELP"),
                HelpTopic::Size => f.write_str("HELP SIZE"),
                HelpTopic::Px => f.write_str("HELP PX"),
            },
            Request::GetSize => f.write_str("SIZE"),
            Request::GetPixel { x, y } => f.write_fmt(format_args!("PX {} {}", x, y)),
            Request::SetPixel { x, y, color } => f.write_fmt(format_args!("PX {} {} {:X}", x, y, color)),
            Request::GetConfig => f.write_str("CONFIG"),
        }
    }
}

/// The response of a pixelflut server
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Response {
    /// Help about a specific topic with more information about that topic
    Help(HelpTopic),
    /// Server configuration information
    ServerConfig(ServerConfig),
    /// Size information about the servers canvas
    Size {
        /// Width of the canvas in number of pixels
        width: usize,
        /// Heigh of the canvas in number of pixels
        height: usize,
    },
    /// Color data of a specific pixel
    PxData {
        /// X coordinate of the pixel
        x: usize,
        /// Y coordinate of the pixel
        y: usize,
        /// The color of the pixel
        color: Color,
    },
}

/// Configuration information about the server
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ServerConfig {
    /// How large a single udp packet is allowed to be.
    pub max_udp_packet_size: usize,
}
