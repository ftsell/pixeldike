//! Data types that describe all protocol interactions as safe-to-use structs

use crate::pixmap::Color;
use std::fmt::{Display, Formatter, Write};

/// The encoding algorithms that the whole canvas can be encoded in
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StateEncodingAlgorithm {
    /// *RGB then Base64* encoding
    Rgb64,
    /// *RGBA then Base64* encoding
    Rgba64,
}

impl Display for StateEncodingAlgorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StateEncodingAlgorithm::Rgb64 => f.write_str("RGB64"),
            StateEncodingAlgorithm::Rgba64 => f.write_str("RGBA64"),
        }
    }
}

/// The help topics that can be requested from the server
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HelpTopic {
    /// Help about the general pixelflut protocol and links to further topics
    General,
    /// Help about the *SIZE* command
    Size,
    /// Help about the *PX* command (both set and get variants)
    Px,
    /// Help about the *STATE* command (including all encodings)
    State,
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
    /// Get the complete canvas in a specific encoding
    GetState(StateEncodingAlgorithm),
}

impl Display for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Request::Help(topic) => match topic {
                HelpTopic::General => f.write_str("HELP"),
                HelpTopic::State => f.write_str("HELP STATE"),
                HelpTopic::Size => f.write_str("HELP SIZE"),
                HelpTopic::Px => f.write_str("HELP PX"),
            },
            Request::GetSize => f.write_str("SIZE"),
            Request::GetPixel { x, y } => f.write_fmt(format_args!("PX {} {}", x, y)),
            Request::SetPixel { x, y, color } => f.write_fmt(format_args!("PX {} {} #{:X}", x, y, color)),
            Request::GetState(alg) => f.write_fmt(format_args!("STATE {} ...", alg)),
            Request::GetConfig => f.write_str("CONFIG"),
        }
    }
}

/// The response of a pixelflut server
#[derive(Debug, Eq, PartialEq)]
pub enum Response<'data> {
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
    /// State of the complete canvas in a specific encoding algorithm
    State {
        /// The algorithm with which the canvas is encoded
        alg: StateEncodingAlgorithm,
        /// Data describing the canvas encoded using `alg`
        data: &'data [u8],
    },
}

/// Configuration information about the server
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ServerConfig {
    /// How large a single udp packet is allowed to be.
    pub max_udp_packet_size: usize,
}

impl Response<'_> {
    /// Transform the response into an owned version using an allocation.
    pub fn to_owned(&self) -> OwnedResponse {
        match self {
            Response::Help(topic) => OwnedResponse::Help(*topic),
            Response::ServerConfig(config) => OwnedResponse::ServerConfig(*config),
            Response::Size { width, height } => OwnedResponse::Size {
                width: *width,
                height: *height,
            },
            Response::PxData { x, y, color } => OwnedResponse::PxData {
                x: *x,
                y: *y,
                color: *color,
            },
            Response::State { alg, data } => OwnedResponse::State {
                alg: *alg,
                data: Vec::from(*data),
            },
        }
    }
}

/// An owned version of `Response`.
///
/// See [`Response`] for detailed description of the enum variants and their fields.
#[allow(missing_docs)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OwnedResponse {
    Help(HelpTopic),
    ServerConfig(ServerConfig),
    Size {
        width: usize,
        height: usize,
    },
    PxData {
        x: usize,
        y: usize,
        color: Color,
    },
    State {
        alg: StateEncodingAlgorithm,
        data: Vec<u8>,
    },
}
