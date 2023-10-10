use crate::net::stream::MsgReader;
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncReadExt};

/// A struct for implementing ReadStream on top of a generic AsyncRead stream
pub(super) struct BufferedMsgReader<R, const CAPACITY: usize = 256>
where
    R: AsyncRead + Unpin + Send,
{
    reader: R,
    read_buffer: [u8; CAPACITY],
    fill_marker: usize,
    msg_marker: usize,
}

impl<R, const CAPACITY: usize> BufferedMsgReader<R, CAPACITY>
where
    R: AsyncRead + Unpin + Send,
{
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            read_buffer: [0; CAPACITY],
            fill_marker: 0,
            msg_marker: 0,
        }
    }
}

#[async_trait]
impl<R, const CAPACITY: usize> MsgReader for BufferedMsgReader<R, CAPACITY>
where
    R: AsyncRead + Unpin + Send,
{
    async fn read_message(&mut self) -> std::io::Result<&[u8]> {
        // reset the buffer
        self.read_buffer[..self.fill_marker].rotate_left(self.msg_marker);
        self.fill_marker -= self.msg_marker;
        self.msg_marker = 0;

        loop {
            // if a valid frame separator (\n) is part of the buffer, return the content before that
            if let Some((i, _)) = self.read_buffer[..self.fill_marker]
                .iter()
                .enumerate()
                .find(|(_, &c)| c == '\n' as u8)
            {
                // return everything up until the found \n as the message (while excluding the \n itself)
                self.msg_marker = i + 1;
                return Ok(&self.read_buffer[..i]);
            }

            // abort if the buffer has already been filled completely
            if self.fill_marker == self.read_buffer.len() {
                return Err(std::io::Error::from(std::io::ErrorKind::OutOfMemory));
            }

            // read new bytes into the buffer
            let bytes_read = self
                .reader
                .read(&mut self.read_buffer[self.fill_marker..])
                .await?;
            if bytes_read != 0 {
                self.fill_marker += bytes_read;
            } else {
                return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
            }
        }
    }
}
