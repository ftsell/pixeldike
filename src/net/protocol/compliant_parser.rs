//! A pixelflut request parser implementation that is fully compliant to the wire protocol

use anyhow::anyhow;
use thiserror::Error;

use crate::net::protocol::{HelpTopic, Request, Response};
use crate::pixmap::Color;

/// Errors that can occur while parsing an input buffer
#[derive(Debug, Error, Copy, Clone, Eq, PartialEq)]
pub enum ParseErr {
    /// The passed pixelflut command is unknown
    #[error("Unknown Command")]
    UnknownCommand,
    /// The passed pixelflut command is known but its invocation was invalid
    #[error("Invalid Command Invocation")]
    InvalidCommand,
}

/// Parse the arguments to a PxSet command
#[inline(always)]
fn parse_px_set_args(x: &str, y: &str, px: &str) -> Result<Request, ParseErr> {
    let xres = x.parse();
    let yres = y.parse();
    let cres = u32::from_str_radix(px, 16);
    match (xres, yres, cres) {
        (Ok(x), Ok(y), Ok(color)) => Ok(Request::SetPixel {
            x,
            y,
            color: Color::from(color),
        }),
        (_, _, _) => Err(ParseErr::UnknownCommand),
    }
}

/// Parse the arguments to a PxGet command
#[inline(always)]
fn parse_px_get_args(x: &str, y: &str) -> Result<Request, ParseErr> {
    let xres = x.parse();
    let yres = y.parse();
    match (xres, yres) {
        (Ok(x), Ok(y)) => Ok(Request::GetPixel { x, y }),
        (_, _) => Err(ParseErr::UnknownCommand),
    }
}

/// Parse the arguments to a Help command
#[inline(always)]
fn parse_help_args(token: &str) -> Result<Request, ParseErr> {
    match token {
        "help" | "HELP" | "general" | "GENERAL" => Ok(Request::Help(HelpTopic::General)),
        "size" | "SIZE" => Ok(Request::Help(HelpTopic::Size)),
        "px" | "PX" => Ok(Request::Help(HelpTopic::Px)),
        _ => Err(ParseErr::InvalidCommand),
    }
}

/// Parse the data part of a PxData response
#[inline(always)]
fn parse_px_data(x: &str, y: &str, px: &str) -> Result<Response, ParseErr> {
    let xres = x.parse();
    let yres = y.parse();
    let cres = u32::from_str_radix(px, 16);
    match (xres, yres, cres) {
        (Ok(x), Ok(y), Ok(color)) => Ok(Response::PxData {
            x,
            y,
            color: Color::from(color),
        }),
        (_, _, _) => Err(ParseErr::UnknownCommand),
    }
}

#[inline(always)]
fn parse_size_data(width: &str, height: &str) -> Result<Response, ParseErr> {
    let width = width.parse();
    let height = height.parse();
    match (width, height) {
        (Ok(width), Ok(height)) => Ok(Response::Size { width, height }),
        (_, _) => Err(ParseErr::InvalidCommand),
    }
}

#[inline(always)]
fn parse_help_data(topic: &str) -> Result<Response, ParseErr> {
    match topic {
        "help" | "HELP" | "general" | "GENERAL" => Ok(Response::Help(HelpTopic::General)),
        "size" | "SIZE" => Ok(Response::Help(HelpTopic::Size)),
        "px" | "PX" => Ok(Response::Help(HelpTopic::Px)),
        _ => Err(ParseErr::InvalidCommand),
    }
}

/// A statically sized buffer containing input tokens.
///
/// This is useful during parsing because it can be allocated on the stack instead of the heap as a Vec would.
struct TokBuf<'s, const MAX_TOKS: usize> {
    /// Storage for up to 4 input tokens
    tokens: [Option<&'s str>; MAX_TOKS],
    /// How many tokens are actually present in the buffer
    len: usize,
}

impl<'s, const MAX_TOKS: usize> TokBuf<'s, MAX_TOKS> {
    #[inline(always)]
    fn tokens(&self) -> &[&'s str] {
        debug_assert_eq!(self.len, self.tokens.iter().filter(|i| i.is_some()).count());
        // Safety: Option is repr(transparent) and we know how many of them are a Some variant
        unsafe { std::mem::transmute(&self.tokens[0..self.len]) }
    }
}

impl<'s, const MAX_TOKS: usize> FromIterator<&'s str> for TokBuf<'s, MAX_TOKS> {
    #[inline(always)]
    fn from_iter<T: IntoIterator<Item = &'s str>>(iter: T) -> Self {
        let mut this = Self {
            tokens: [None; MAX_TOKS],
            len: 0,
        };

        for (i, token) in iter.into_iter().take(MAX_TOKS).enumerate() {
            this.tokens[i] = Some(token);
            this.len += 1;
        }

        this
    }
}

/// Try to parse a single pixelflut request
#[inline(always)]
pub fn parse_request_str(line: &str) -> Result<Request, ParseErr> {
    let tokens: TokBuf<'_, 4> = line.split_whitespace().collect();
    let tokens = tokens.tokens();
    match tokens.len() {
        4 => parse_px_set_args(tokens[1], tokens[2], tokens[3]),
        3 => parse_px_get_args(tokens[1], tokens[2]),
        2 => parse_help_args(tokens[1]),
        1 => match tokens[0] {
            "SIZE" | "size" => Ok(Request::GetSize),
            "HELP" | "help" => Ok(Request::Help(HelpTopic::General)),
            _ => Err(ParseErr::UnknownCommand),
        },
        0 => Err(ParseErr::InvalidCommand),
        _ => unreachable!(),
    }
}

/// Parse a single request from a byte slice
#[inline(always)]
pub fn parse_request_bin(line: &[u8]) -> anyhow::Result<Request> {
    if line.is_ascii() {
        // Safety: This is fine because the bytes are already checked to be ascii
        let str = unsafe { std::str::from_utf8_unchecked(line) };
        Ok(parse_request_str(str)?)
    } else {
        Err(anyhow!("request buffer does not contain an ascii string"))
    }
}

/// Try to parse a single pixelflut response
#[inline(always)]
pub fn parse_response_str(line: &str) -> Result<Response, ParseErr> {
    let tokens: TokBuf<'_, 4> = line.split_whitespace().collect();
    let tokens = tokens.tokens();
    match tokens.len() {
        4 => parse_px_data(tokens[1], tokens[2], tokens[3]),
        3 => parse_size_data(tokens[1], tokens[2]),
        2 => parse_help_data(tokens[1]),
        _ => Err(ParseErr::UnknownCommand),
    }
}

/// Parse a single pixelflut response from a byte slice
#[inline(always)]
pub fn parse_response_bin(line: &[u8]) -> anyhow::Result<Response> {
    if line.is_ascii() {
        // Safety: This is fine because the bytes are already checked to be ascii
        let str = unsafe { std::str::from_utf8_unchecked(line) };
        Ok(parse_response_str(str)?)
    } else {
        Err(anyhow!("response buffer does not contain an ascii string"))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ::test::Bencher;
    use std::hint::black_box;

    #[test]
    fn test_parse_commands() {
        fn run_test(line: &str, res: Request) {
            let req = parse_request_str(line);
            assert_eq!(req, Ok(res), "{:06x?} != Ok({:06x?})", req, res);
        }

        run_test("HELP", Request::Help(HelpTopic::General));
        run_test("SIZE", Request::GetSize);
        run_test(
            "PX 42 128 AABBCC",
            Request::SetPixel {
                x: 42,
                y: 128,
                color: Color::from((0xAA, 0xBB, 0xCC)),
            },
        );
        run_test(
            "PX 0 0 AABBCC",
            Request::SetPixel {
                x: 0,
                y: 0,
                color: Color::from((0xAA, 0xBB, 0xCC)),
            },
        );
    }

    #[bench]
    fn bench_parse_get_pixel(b: &mut Bencher) {
        let cmd = black_box("PX 17 7632");
        b.iter(move || parse_request_str(cmd).unwrap());
    }

    #[bench]
    fn bench_parse_set_pixel(b: &mut Bencher) {
        let cmd = "PX 17 7632 12FBA5";
        b.iter(move || parse_request_str(black_box(cmd)).unwrap());
    }

    #[bench]
    fn bench_parse_size(b: &mut Bencher) {
        let cmd = "SIZE";
        b.iter(move || parse_request_str(black_box(cmd)).unwrap());
    }

    /*
    pub fn read_request_slice<'b, 'r: 'b>(
        read: &'r mut impl BufRead,
        buf: &'b mut Vec<u8>,
    ) -> std::io::Result<Option<&'b str>> {
        // Clear previous line, if any, but keep capacity
        buf.clear();

        // This constant is important, because otherwise
        // a malicious client could exhaust our memory by never sending
        // a newline.
        const MAX_LINE_LENGTH: usize = 32;
        let read = read.take(MAX_LINE_LENGTH as u64).read_until('\n' as u8, buf)?;
        match read {
            0 => Ok(None),
            MAX_LINE_LENGTH => Err(io::Error::new(io::ErrorKind::Other, "MAX_LINE_LENGTH exceeded")),
            _ => {
                let line = &buf[0..read];
                if line.is_ascii() {
                    let str = unsafe { std::str::from_utf8_unchecked(line) };
                    Ok(Some(str))
                } else {
                    Err(io::Error::new(io::ErrorKind::Other, "not an ascii string"))
                }
            }
        }
    }
     */

    /*
    #[inline(always)]
    pub fn read_request_line<'l, 'r: 'l>(
        read: &'r mut impl BufRead,
        line: &'l mut String,
    ) -> std::io::Result<Option<&'l str>> {
        // Clear previous line, if any, but keep capacity
        line.clear();

        // This constant is important, because otherwise
        // a malicious client could exhaust our memory by never sending
        // a newline.
        const MAX_LINE_LENGTH: usize = 32;
        let read = read.take(MAX_LINE_LENGTH as u64).read_line(line)?;
        match read {
            0 => Ok(None),
            MAX_LINE_LENGTH => Err(io::Error::new(io::ErrorKind::Other, "MAX_LINE_LENGTH exceeded")),
            _ => Ok(Some(line.as_str())),
        }
    }
     */

    // pub fn pixmap_consumer(mut read: impl BufRead, pixmap: &mut Pixmap) -> PxResult<()> {
    //     let mut line = String::with_capacity(32);
    //     while let Some(line) = read_request_line(&mut read, &mut line)? {
    //         let req = parse_request_line(line)?;
    //         if let Request::SetPixel { x, y, color } = req {
    //             let idx = x as u32 + y as u32 * pixmap.width;
    //             *pixmap.pixels.get_mut(idx as usize).unwrap() = color;
    //         };
    //     }
    //     Ok(())
    // }
    //
    // pub fn consume(mut read: impl BufRead, mut setpx: impl FnMut(u16, u16, u32)) -> Result<(), Box<dyn Error>> {
    //     let mut line = String::with_capacity(32);
    //     while let Some(line) = read_request_line(&mut read, &mut line)? {
    //         let req = parse_request_line(line)?;
    //         if let Request::SetPixel { x, y, color } = req {
    //             setpx(x, y, color)
    //         };
    //     }
    //     Ok(())
    // }
    //
    // #[allow(dead_code)]
    // fn handle_requests(
    //     mut read: impl BufRead,
    //     mut on_request: impl FnMut(Request),
    // ) -> Result<(), Box<dyn Error>> {
    //     let mut line = String::with_capacity(32);
    //     while let Some(line) = read_request_line(&mut read, &mut line)? {
    //         let req = parse_request_line(line)?;
    //         on_request(req)
    //     }
    //     Ok(())
    // }
    //
    // #[allow(dead_code)]
    // pub fn print_consumer(read: impl BufRead) -> PxResult<()> {
    //     handle_requests(read, |req| println!("{:?}", req))
    // }
    //
    // #[cfg(test)]
    // mod tests {
    //     extern crate test;
    //     use std::io::BufReader;
    //
    //     use crate::CyclicRead;
    //
    //     use super::*;
    //     use std::hint::black_box;
    //     use test::bench::Bencher;
    //
    //     #[bench]
    //     fn bench_line_split_whitespace(b: &mut Bencher) {
    //         let input = black_box("PX 12345 27890 ffDE12");
    //         b.iter(|| input.split_whitespace().last());
    //     }
    //
    //     #[bench]
    //     fn bench_parse_line(b: &mut Bencher) {
    //         let input = black_box("PX  12345 27890 ffDE12");
    //         b.iter(move || parse_request_line(&input).unwrap())
    //     }
    //
    //     #[bench]
    //     fn bench_read_line(b: &mut Bencher) {
    //         let mut line = String::with_capacity(32);
    //         let input = std::fs::read("testcase.txt").expect("no testcase file found");
    //         let mut input = BufReader::new(CyclicRead {
    //             cursor: 0,
    //             bytes: input.as_slice(),
    //         });
    //         b.iter(move || read_request_line(&mut input, &mut line).unwrap().is_some())
    //     }
    //
    //     #[bench]
    //     fn bench_read_line_ascii(b: &mut Bencher) {
    //         let mut line = Vec::with_capacity(32);
    //         let input = std::fs::read("testcase.txt").expect("no testcase file found");
    //         let mut input = BufReader::new(CyclicRead {
    //             cursor: 0,
    //             bytes: input.as_slice(),
    //         });
    //         b.iter(move || read_request_slice(&mut input, &mut line).unwrap().is_some())
    //     }
    //
    //     #[bench]
    //     fn bench_parse_lines(b: &mut Bencher) {
    //         let mut line = String::with_capacity(32);
    //         let input = std::fs::read("testcase.txt").expect("no testcase file found");
    //         let mut input = BufReader::new(CyclicRead {
    //             cursor: 0,
    //             bytes: input.as_slice(),
    //         });
    //         b.iter(move || {
    //             let line = read_request_line(&mut input, &mut line).unwrap().unwrap();
    //             let req = parse_request_line(line).expect("should be valid request");
    //             req
    //         })
    //     }
    //
    //     #[bench]
    //     fn bench_parse_lines_ascii(b: &mut Bencher) {
    //         let mut line = Vec::with_capacity(32);
    //         let input = std::fs::read("testcase.txt").expect("no testcase file found");
    //         let mut input = BufReader::new(CyclicRead {
    //             cursor: 0,
    //             bytes: input.as_slice(),
    //         });
    //         b.iter(move || {
    //             let line = read_request_slice(&mut input, &mut line).unwrap().unwrap();
    //             let req = parse_request_line(line).expect("should be valid request");
    //             req
    //         })
    //     }
    //
    //     #[bench]
    //     fn bench_pixmap_requests(b: &mut Bencher) {
    //         let mut pixmap = Pixmap::new(1920, 1080);
    //         let mut line = String::with_capacity(32);
    //         let input = std::fs::read("testcase.txt").expect("no testcase file found");
    //         let mut input = BufReader::new(CyclicRead {
    //             cursor: 0,
    //             bytes: input.as_slice(),
    //         });
    //         let pixref = &mut pixmap;
    //         b.iter(move || {
    //             let line = read_request_line(&mut input, &mut line).unwrap().unwrap();
    //             let req = parse_request_line(line).expect("should be valid request");
    //             let Request::SetPixel { x, y, color } = req else {
    //                 panic!("not a set pixel request")
    //             };
    //             let idx = x as u32 + y as u32 * pixref.width;
    //             *pixref.pixels.get_mut(idx as usize).unwrap() = color;
    //         });
    //         let sum: usize = pixmap.pixels.iter().map(|&x| x as usize).sum();
    //         println!("sum: {}", sum);
    //     }
    //
    //     #[bench]
    //     fn bench_pixmap_requests_ascii(b: &mut Bencher) {
    //         let mut pixmap = Pixmap::new(1920, 1080);
    //         let mut line = Vec::with_capacity(32);
    //         let input = std::fs::read("testcase.txt").expect("no testcase file found");
    //         let mut input = BufReader::new(CyclicRead {
    //             cursor: 0,
    //             bytes: input.as_slice(),
    //         });
    //         let pixref = &mut pixmap;
    //         b.iter(move || {
    //             let line = read_request_slice(&mut input, &mut line).unwrap().unwrap();
    //             let req = parse_request_line(line).expect("should be valid request");
    //             let Request::SetPixel { x, y, color } = req else {
    //                 panic!("not a set pixel request")
    //             };
    //             let idx = x as u32 + y as u32 * pixref.width;
    //             *pixref.pixels.get_mut(idx as usize).unwrap() = color;
    //         });
    //         let sum: usize = pixmap.pixels.iter().map(|&x| x as usize).sum();
    //         println!("sum: {}", sum);
    //     }
    //
    //     #[allow(dead_code)]
    //     //#[bench]
    //     fn bench_pixmap_consumer(b: &mut Bencher) {
    //         let mut pixmap = Pixmap::new(1920, 1080);
    //         let input = std::fs::read("testcase.txt").expect("no testcase file found");
    //         let pixref = &mut pixmap;
    //         b.iter(move || {
    //             let input = BufReader::new(std::io::Cursor::new(input.as_slice()));
    //             pixmap_consumer(input, pixref).unwrap();
    //         });
    //         let sum: usize = pixmap.pixels.iter().map(|&x| x as usize).sum();
    //         println!("sum: {}", sum);
    //     }
    // }
}
