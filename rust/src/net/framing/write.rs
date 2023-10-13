use crate::i18n;
use crate::net::protocol::{HelpTopic, Request, Response};
use async_trait::async_trait;

/// A trait for structs that support writing pixelflut messages into them
#[async_trait]
pub trait MsgWriter: Send + Sync {
    /// Write some bytes that contain parts of a pixelflut message into the network.
    ///
    /// This is a low level function that does not perform any framing and instead only writes the bytes into the
    /// network as they are handed to it.
    async fn write_data(&mut self, msg: &[u8]) -> std::io::Result<()>;

    /// Flush potentially buffered data into the network.
    async fn flush(&mut self) -> std::io::Result<()>;

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
            Request::GetConfig => self.write_message("CONFIG".as_bytes()).await,
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
            Response::ServerConfig(config) => {
                let msg = format!("CONFIG max_udp_packet_size={}", config.max_udp_packet_size);
                self.write_message(msg.as_bytes()).await
            }
        }
    }
}

/// A `MsgWriter` implementation that writes messages into the void as a noop.
#[derive(Debug, Copy, Clone)]
pub struct VoidWriter;

#[async_trait]
impl MsgWriter for VoidWriter {
    async fn write_data(&mut self, _msg: &[u8]) -> std::io::Result<()> {
        Ok(())
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
