use bytes::Bytes;
use std::io::Cursor;
use std::str::Utf8Error;

pub(crate) enum Error {
    Incomplete,
    Utf8Error(Utf8Error),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) enum Frame {
    Simple(String),
}

impl Frame {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        Frame::get_line(src).and_then(Frame::get_string).map(|_| ())
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Self, Error> {
        Frame::get_line(src)
            .and_then(Frame::get_string)
            .map(|s| Frame::Simple(String::from(s)))
    }

    pub fn encode(self) -> Bytes {
        match self {
            Frame::Simple(s) => {
                let mut s = s.into_bytes();
                s.append(&mut vec![b'\n']);
                s.into()
            }
        }
    }

    fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
        let start = src.position() as usize;
        let end = src.get_ref().len();
        let b: Vec<u8> = src.get_ref()[start..end].iter().cloned().collect();

        // try to find a complete line in the buffer
        match (start..end).find_map(|i| {
            if src.get_ref()[i] == b'\n' || src.get_ref()[i] == b'\r' {
                src.set_position((i + 1) as u64);
                Some(&src.get_ref()[start..i])
            } else {
                None
            }
        }) {
            None => Err(Error::Incomplete),
            Some(line) => Ok(line),
        }
    }

    fn get_string(src: &[u8]) -> Result<&str, Error> {
        match std::str::from_utf8(src) {
            Err(e) => Err(Error::Utf8Error(e)),
            Ok(s) => Ok(s),
        }
    }
}

impl Into<Bytes> for Frame {
    fn into(self) -> Bytes {
        self.encode()
    }
}
