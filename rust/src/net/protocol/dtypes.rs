//! Data types that describe all protocol interactions as safe-to-use structs

use crate::i18n;
use crate::pixmap::Color;
use std::fmt::{Display, Formatter};
use std::io::Write;
use tokio::io::{AsyncWrite, AsyncWriteExt};

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

impl Request {
    /// Write the binary representation of this request into the given writer
    pub fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Request::Help(topic) => match topic {
                HelpTopic::General => writer.write_all("HELP\n".as_bytes()),
                HelpTopic::Size => writer.write_all("HELP SIZE\n".as_bytes()),
                HelpTopic::Px => writer.write_all("HELP PX\n".as_bytes()),
            },
            Request::GetSize => writer.write_all("SIZE\n".as_bytes()),
            Request::GetPixel { x, y } => writer.write_all(format!("PX {} {}\n", x, y).as_bytes()),
            Request::SetPixel { x, y, color } => {
                writer.write_all(format!("PX {} {} {:X}\n", x, y, color).as_bytes())
            }
        }
    }

    /// Write the binary representation of this request into the given async writer
    pub async fn write_async(&self, writer: &mut (impl AsyncWrite + Unpin)) -> std::io::Result<()> {
        match self {
            Request::Help(topic) => match topic {
                HelpTopic::General => writer.write_all("HELP\n".as_bytes()).await,
                HelpTopic::Size => writer.write_all("HELP SIZE\n".as_bytes()).await,
                HelpTopic::Px => writer.write_all("HELP PX\n".as_bytes()).await,
            },
            Request::GetSize => writer.write_all("SIZE\n".as_bytes()).await,
            Request::GetPixel { x, y } => writer.write_all(format!("PX {} {}\n", x, y).as_bytes()).await,
            Request::SetPixel { x, y, color } => {
                writer
                    .write_all(format!("PX {} {} {:X}\n", x, y, color).as_bytes())
                    .await
            }
        }
    }
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
        }
    }
}

/// The response of a pixelflut server
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Response {
    /// Help about a specific topic with more information about that topic
    Help(HelpTopic),
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

impl Response {
    /// Write the binary representation of this response into the given writer
    pub fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            Response::Help(topic) => match topic {
                HelpTopic::General => writer.write_all(i18n::HELP_GENERAL.as_bytes()),
                HelpTopic::Size => writer.write_all(i18n::HELP_SIZE.as_bytes()),
                HelpTopic::Px => writer.write_all(i18n::HELP_PX.as_bytes()),
            },
            Response::Size { width, height } => {
                writer.write_all(format!("SIZE {} {}\n", width, height).as_bytes())
            }
            Response::PxData { x, y, color } => {
                writer.write_all(format!("PX {} {} {:X}\n", x, y, color).as_bytes())
            }
        }
    }

    /// Write the binary representation of this response into the given async writer
    pub async fn write_async(&self, writer: &mut (impl AsyncWrite + Unpin)) -> std::io::Result<()> {
        match self {
            Response::Help(topic) => match topic {
                HelpTopic::General => writer.write_all(i18n::HELP_GENERAL.as_bytes()).await,
                HelpTopic::Size => writer.write_all(i18n::HELP_SIZE.as_bytes()).await,
                HelpTopic::Px => writer.write_all(i18n::HELP_PX.as_bytes()).await,
            },
            Response::Size { width, height } => {
                writer
                    .write_all(format!("SIZE {} {}\n", width, height).as_bytes())
                    .await
            }
            Response::PxData { x, y, color } => {
                writer
                    .write_all(format!("PX {} {} {:X}\n", x, y, color).as_bytes())
                    .await
            }
        }
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Response::Help(topic) => match topic {
                HelpTopic::General => f.write_str(i18n::HELP_GENERAL),
                HelpTopic::Size => f.write_str(i18n::HELP_SIZE),
                HelpTopic::Px => f.write_str(i18n::HELP_PX),
            },
            Response::Size { width, height } => f.write_fmt(format_args!("SIZE {} {}", width, height)),
            Response::PxData { x, y, color } => f.write_fmt(format_args!("PX {} {} {:X}", x, y, color)),
        }
    }
}
