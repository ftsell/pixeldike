use crate::net::MsgReader;
use async_trait::async_trait;
use std::ops::DerefMut;

/// A struct that stores a fixed buffer and implements ReadStream on top of it
pub(super) struct FixedMsgStream<const CAPACITY: usize> {
    /// The fixed buffer that may contain messages
    buffer: [u8; CAPACITY],
    /// A marker to indicate up to which points messages have already been read
    msg_marker: usize,
}

impl<const CAPACITY: usize> FixedMsgStream<CAPACITY> {
    pub fn new() -> Self {
        Self {
            buffer: [0; CAPACITY],
            msg_marker: 0,
        }
    }

    pub fn get_buf_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}

#[async_trait]
impl<const CAPACITY: usize> MsgReader for FixedMsgStream<CAPACITY> {
    async fn read_message(&mut self) -> std::io::Result<&[u8]> {
        // if a valid frame separator (\n) is part of the buffer, return the content before that
        if let Some((i, _)) = self.buffer[self.msg_marker..]
            .iter()
            .enumerate()
            .find(|(_, &c)| c == '\n' as u8)
        {
            // return everything up until the found \n as the message (while excluding the \n itself)
            self.msg_marker = i + 1;
            return Ok(&self.buffer[..i]);
        }

        Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof))
    }
}
