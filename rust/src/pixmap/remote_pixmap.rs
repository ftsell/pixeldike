use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::ops::DerefMut;
use std::sync::Mutex;

use crate::net::{MsgReader, MsgWriter};
use crate::net_protocol::{parse_response, OwnedResponse, Request, Response};
use anyhow::{Error, Result};
use tokio::io::{AsyncBufReadExt, AsyncRead};
use tokio::runtime::Runtime;
use tokio::task::block_in_place;

use crate::state_encoding;

use super::traits::*;
use super::*;

/// A pixmap implementation that proxies through to another pixelflut server.
///
/// It is implemented on generic [`Read`] and [`Write`] trait requirements but these are expected
/// to be a communication channel to something that speaks the pixelflut protocol.
#[derive(Debug)]
pub struct RemotePixmap<R, W>
where
    R: MsgReader,
    W: MsgWriter,
{
    net: Mutex<(R, W)>,
    width: usize,
    height: usize,
}

impl<R, W> RemotePixmap<R, W>
where
    R: MsgReader,
    W: MsgWriter,
{
    /// Create a new instance by using the given *reader* and *writer* implementations as a
    /// communication channel.
    ///
    /// The resulting instance will write pixelflut [`OldFrame`]s into *writer* and expects
    /// corresponding [`OldFrame`] responses by reading from *reader*.
    /// An example is to use a TCP socket connected to another pixelflut server and pass
    /// the [`TcpStream`](std::net::TcpStream) (after calling [`try_clone`](std::net::TcpStream::try_clone)) as both
    /// *reader* and *writer*.
    pub fn new(reader: R, writer: W) -> Result<Self> {
        let mut instance = Self {
            net: Mutex::new((reader, writer)),
            width: 0,
            height: 0,
        };
        instance.fetch_size()?;

        Ok(instance)
    }

    async fn send(&self, request: &Request) -> Result<()> {
        let mut lock = self.net.lock().unwrap();
        let (_reader, writer) = lock.deref_mut();

        // send request
        tracing::debug!("Sending {:?}", request);
        writer.write_request(&request).await.unwrap();
        writer.flush().await.unwrap();
        Ok(())
    }

    async fn receive(&self) -> Result<OwnedResponse> {
        let response = {
            let mut lock = self.net.lock().unwrap();
            let response = lock.0.read_message().await?;
            let (_, response) = parse_response(&response)?;
            response.to_owned()
        };
        Ok(response)
    }

    async fn fetch_size(&mut self) -> Result<()> {
        self.send(&Request::GetSize).await?;
        let response = self.receive().await?;

        match response {
            OwnedResponse::Size { width, height } => {
                self.width = width;
                self.height = height;

                Ok(())
            }
            _ => Err(Error::msg(format!(
                "invalid response '{:?}' for SIZE request",
                response
            ))),
        }
    }
}

impl<R, W> PixmapBase for RemotePixmap<R, W>
where
    R: MsgReader,
    W: MsgWriter,
{
    fn get_size(&self) -> Result<(usize, usize)> {
        Ok((self.width, self.height))
    }
}

impl<R, W> PixmapRead for RemotePixmap<R, W>
where
    R: MsgReader,
    W: MsgWriter,
{
    fn get_pixel(&self, x: usize, y: usize) -> Result<Color> {
        block_in_place(move || {
            tokio::runtime::Handle::current().block_on(async {
                self.send(&Request::GetPixel { x, y }).await?;
                let response = self.receive().await?;
                match response {
                    Response::PxData { x, y, color } => Ok(color),
                    _ => Err(Error::msg(format!(
                        "Received invalid response for GetPixel request: {:?}",
                        response
                    ))),
                }
            })
        })
    }
}

impl<R, W> PixmapWrite for RemotePixmap<R, W>
where
    R: MsgReader,
    W: MsgWriter,
{
    fn set_pixel(&self, x: usize, y: usize, color: Color) -> Result<()> {
        block_in_place(move || {
            tokio::runtime::Handle::current()
                .block_on(async { self.send(&Request::SetPixel { x, y, color }).await })
        })
    }
}

#[cfg(test)]
mod test {}
