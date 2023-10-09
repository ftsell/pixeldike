use crate::i18n;
use crate::net_protocol::{HelpTopic, Request, Response};
use async_trait::async_trait;
use tokio::io::{AsyncWrite, AsyncWriteExt};

/// An abstraction over the network layer to which pixelflut messages can be written.
///
/// # Implementors Note
/// When implementing this trait, it is usually enough to implement `write_data()`.
#[async_trait]
pub trait MsgWriter: Send + Sync {
    /// Write some bytes that contain parts of a pixelflut message into the network.
    ///
    /// This is a low level function that does not perform any framing and instead only writes the bytes into the
    /// network as they are handed to it.
    async fn write_data(&mut self, msg: &[u8]) -> std::io::Result<()>;

    /// Write a complete pixelflut message into the network.
    async fn write_message(&mut self, msg: &[u8]) -> std::io::Result<()> {
        self.write_data(msg).await?;
        self.write_message_delimiter().await
    }

    /// Write the message delimiter (\n) into the network.
    async fn write_message_delimiter(&mut self) -> std::io::Result<()> {
        self.write_data(&['\n' as u8]).await
    }

    /// Encode a request and write it to the network.
    async fn write_request(&mut self, request: &Request) -> std::io::Result<()> {
        match request {
            Request::Help(topic) => match topic {
                HelpTopic::General => self.write_message("HELP".as_bytes()).await,
                HelpTopic::Size => self.write_message("HELP SIZE".as_bytes()).await,
                HelpTopic::Px => self.write_message("HELP PX".as_bytes()).await,
                HelpTopic::State => self.write_message("HELP STATE".as_bytes()).await,
            },
            Request::GetSize => self.write_message("SIZE".as_bytes()).await,
            Request::GetPixel { x, y } => {
                let msg = format!("PX {} {}", x, y);
                self.write_message(msg.as_bytes()).await
            }
            Request::SetPixel { x, y, color } => {
                let msg = format!("PX {} {} #{:X}", x, y, color);
                self.write_message(msg.as_bytes()).await
            }
            Request::GetState(alg) => {
                let msg = format!("STATE {}", alg);
                self.write_message(msg.as_bytes()).await
            }
        }
    }

    /// Encode a response and write it to the network.
    async fn write_response(&mut self, response: &Response) -> std::io::Result<()> {
        match response {
            Response::Help(topic) => match topic {
                HelpTopic::General => self.write_message(i18n::HELP_GENERAL.as_bytes()).await,
                HelpTopic::Size => self.write_message(i18n::HELP_SIZE.as_bytes()).await,
                HelpTopic::Px => self.write_message(i18n::HELP_PX.as_bytes()).await,
                HelpTopic::State => self.write_message(i18n::HELP_STATE.as_bytes()).await,
            },
            Response::Size { width, height } => {
                let msg = format!("SIZE {} {}", width, height);
                self.write_message(msg.as_bytes()).await
            }
            Response::PxData { x, y, color } => {
                let msg = format!("PX {} {} #{:X}", x, y, color);
                self.write_message(msg.as_bytes()).await
            }
            Response::State { alg, data } => {
                self.write_data(format!("STATE {} ", alg).as_bytes()).await?;
                self.write_data(data).await?;
                self.write_message_delimiter().await
            }
        }
    }
}

#[async_trait]
impl<W> MsgWriter for W
where
    W: AsyncWrite + Send + Sync + Unpin,
{
    async fn write_data(&mut self, msg: &[u8]) -> std::io::Result<()> {
        self.write(msg).await?;
        Ok(())
    }
}

/// An abstraction over the network layer from which pixelflut messages can be read.
#[async_trait]
pub trait MsgReader {
    /// Read an encoded pixelflut message from the network.
    async fn read_message(&mut self) -> std::io::Result<&[u8]>;
}
