use anyhow::anyhow;
use async_trait::async_trait;

/// A trait for objects which can refill a buffer from the network
#[async_trait]
pub trait BufferFiller: Sized {
    /// Fill the given buffer with new data, returning how many bytes were added
    async fn fill_buffer(&mut self, buffer: &mut [u8]) -> anyhow::Result<usize>;
}

/// An implementation of [`BufferFiller`] that doesn't actually fill any data into the buffer
#[derive(Debug, Copy, Clone)]
pub struct NullFiller;

#[async_trait]
impl BufferFiller for NullFiller {
    async fn fill_buffer(&mut self, _buffer: &mut [u8]) -> anyhow::Result<usize> {
        Err(anyhow!("NullFiller cannot refill buffers"))
    }
}

/// A struct for reading pixelflut messages from an internal buffer
#[derive(Debug, Eq, PartialEq)]
pub struct BufferedMsgReader<const BUF_SIZE: usize, T: BufferFiller> {
    /// The buffer which contains potential message data
    buffer: [u8; BUF_SIZE],
    /// A marker indexing into the buffer indicating up to which points messages have already been read by callers
    read_msg_marker: usize,
    /// A marker indexing into the buffer indicating to where the buffer is full of data
    fill_marker: usize,
    refiller: T,
}

impl<const BUF_SIZE: usize, T: BufferFiller> BufferedMsgReader<BUF_SIZE, T> {
    /// Create a new `BufferedMsgReader` with empty content that uses the given refiller to refill its buffer
    /// when it is empty.
    pub fn new_empty(refiller: T) -> Self {
        Self {
            buffer: [0; BUF_SIZE],
            read_msg_marker: 0,
            fill_marker: 0,
            refiller,
        }
    }
}

impl<const BUF_SIZE: usize, T: BufferFiller> BufferedMsgReader<BUF_SIZE, T> {
    /// Read a message from the internal buffer if there is one and advance the marker so that the message is not
    /// read again.
    ///
    /// If the buffer does not contain enough data it is automatically refilled again from the refiller.
    pub async fn read_msg(&mut self) -> anyhow::Result<&[u8]> {
        // shift out all already read messages from the buffer
        self.buffer[..self.fill_marker].rotate_left(self.read_msg_marker);
        self.fill_marker -= self.read_msg_marker;
        self.read_msg_marker = 0;

        loop {
            // return a message from the buffer if there is an unread one still in it
            if let Some((separator_pos, _)) = self.buffer[..self.fill_marker]
                .iter()
                .enumerate()
                .find(|(_, &c)| c == b'\n')
            {
                self.read_msg_marker = separator_pos + 1;
                return Ok(&self.buffer[..separator_pos]);
            }

            // abort if the buffer has already been filled completely
            if self.fill_marker == self.buffer.len() {
                return Err(anyhow!(
                    "buffer has been filled completely but it contains no messages"
                ));
            }

            // read new bytes into the buffer
            self.fill_marker += self
                .refiller
                .fill_buffer(&mut self.buffer[self.fill_marker..])
                .await?;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_read_multiple_commands() {
        struct StaticFiller<'a> {
            already_read: bool,
            content: &'a [u8],
        }
        #[async_trait]
        impl BufferFiller for StaticFiller<'_> {
            async fn fill_buffer(&mut self, buffer: &mut [u8]) -> anyhow::Result<usize> {
                if !self.already_read {
                    self.already_read = true;
                    assert!(buffer.len() >= self.content.len());
                    buffer[..self.content.len()].copy_from_slice(self.content);
                    Ok(self.content.len())
                } else {
                    Err(anyhow!("StaticReader content has already been read"))
                }
            }
        }

        let static_filler = StaticFiller {
            already_read: false,
            content: "HELP\nHELP\n".as_bytes(),
        };
        let mut reader = BufferedMsgReader::<32, _>::new_empty(static_filler);
        {
            let content1 = reader.read_msg().await.unwrap();
            assert_eq!(content1, "HELP".as_bytes());
        }
        {
            let content2 = reader.read_msg().await.unwrap();
            assert_eq!(content2, "HELP".as_bytes());
        }
    }
}
