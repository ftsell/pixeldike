//!
//! Framing is the process of taking a byte stream and converting it to a stream of frames.
//! A frame is a unit of data transmitted between two peers.
//!

use crate::protocol::{Request, Response};
use anyhow::{Error, Result};
use bytes::buf::Take;
use bytes::{Buf, Bytes};
use std::convert::TryInto;
use std::ops::{Deref, DerefMut};

/// A frame is a unit of data transmitted between two peers.
pub struct Frame<I>(I)
where
    I: Buf;

impl<I> AsRef<I> for Frame<I>
where
    I: Buf,
{
    fn as_ref(&self) -> &I {
        &self.0
    }
}

impl<I> AsMut<I> for Frame<I>
where
    I: Buf,
{
    fn as_mut(&mut self) -> &mut I {
        &mut self.0
    }
}

impl<I> Deref for Frame<I>
where
    I: Buf,
{
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I> DerefMut for Frame<I>
where
    I: Buf,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// TODO Can this be done for TryInto<&str>?
impl<I> TryInto<String> for Frame<I>
where
    I: Buf,
{
    type Error = Error;

    fn try_into(mut self) -> Result<String, Self::Error> {
        let length = self.remaining();
        String::from_utf8(self.copy_to_bytes(length).to_vec())
            .map_err(|_e| Error::msg("utf8 error while decoding frame"))
    }
}

impl From<Request> for Frame<Bytes> {
    fn from(src: Request) -> Self {
        Self(Bytes::from(src.to_string()))
    }
}

impl From<Response> for Frame<Bytes> {
    fn from(src: Response) -> Self {
        Self(Bytes::from(src.to_string()))
    }
}

impl<I> Frame<I>
where
    I: Buf,
{
    pub fn encode(self) -> Bytes {
        let mut chain = self.0.chain(Bytes::from_static("\n".as_bytes()));
        let length = chain.remaining();
        chain.copy_to_bytes(length)
    }
}

impl<I> Frame<I>
where
    I: Buf + Clone,
{
    /// Try to extract a Frame from an input buffer.
    /// If successful, returns the extracted framed as well as how many bytes were read to extract it.
    pub fn from_input(src: I) -> Result<(Frame<Take<I>>, usize)> {
        // use a separate peeking view into the buffer to search for a newline
        let mut peeker = src.clone();
        while peeker.has_remaining() {
            let b = peeker.get_u8();
            if b == '\n' as u8 || b == '\r' as u8 {
                // construct a buffer view which is limited to the found line
                let length = src.remaining() - peeker.remaining();
                return Ok((Frame(src.take(length)), length));
            }
        }

        Err(Error::msg("input is not a complete frame"))
    }
}

impl Frame<Bytes> {
    pub fn new_from_string(content: String) -> Self {
        Self(Bytes::from(content))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::TestResult;

    quickcheck! {
        fn test_parsing_encoding_stay_the_same(input: String) -> TestResult {
            if input.contains("\n") || input.contains("\r") {
                return TestResult::discard();
            }

            let input = input + "\n";
            let input_bytes = input.into_bytes();

            match Frame::parse(&mut input_bytes) {
                Err(_) => TestResult::discard(),
                Ok(frame) => TestResult::from_bool(frame.encode() == input_bytes)
            }
        }
    }

    #[test]
    fn test_no_termination_character() {
        let result = Frame::parse(&mut "abc123".as_bytes());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), super::Error::Incomplete);
    }
}
