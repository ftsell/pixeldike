use crate::net::framing::{BufferFiller, BufferedMsgReader, MsgWriter};
use async_trait::async_trait;

/// A trait to unify the different transport protocol clients
#[async_trait]
pub trait GenClient<const READ_BUF_SIZE: usize>: Sized {
    /// An options type with which the client can be configured.
    type Options;

    /// An associated type that can be used to read messages from the client
    type MsgWriter: MsgWriter;

    type BufferFiller: BufferFiller;

    /// Create a new client by connecting to a pixelflut server.
    async fn connect(options: Self::Options) -> anyhow::Result<Self>;

    /// Get a `MsgWriter` implementation that sends messages through this client
    fn get_msg_writer(&mut self) -> &mut Self::MsgWriter;

    fn get_msg_reader(&mut self) -> &mut BufferedMsgReader<READ_BUF_SIZE, Self::BufferFiller>;
}
