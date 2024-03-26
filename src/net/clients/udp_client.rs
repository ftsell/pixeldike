use crate::net::protocol::{parse_response_bin, Request, Response};
use anyhow::anyhow;
use bytes::{BufMut, BytesMut};
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::net::UdpSocket;

/// A pixelflut client that uses UDP for communication with a pixelflut server.
///
/// Not that requests are not buffered or assembled into larger UDP packets in any way.
/// Instead, every request is sent as its own datagram which is very inefficient.
#[derive(Debug)]
pub struct UdpClient {
    socket: UdpSocket,
}

impl UdpClient {
    /// Try to connect to the server running at the given address
    pub async fn connect(addr: &SocketAddr) -> std::io::Result<Self> {
        let socket = if addr.is_ipv4() {
            UdpSocket::bind(SocketAddr::from_str("0.0.0.0:0").unwrap()).await?
        } else {
            UdpSocket::bind(SocketAddr::from_str("[::]:0").unwrap()).await?
        };
        socket.connect(addr).await?;
        Ok(Self { socket })
    }

    /// Send a single request to the configured server
    pub async fn send_request(&mut self, request: Request) -> std::io::Result<()> {
        let mut buf = BytesMut::with_capacity(64).writer();
        request.write(&mut buf).unwrap();
        self.socket.send(&buf.get_ref()).await?;
        Ok(())
    }

    /// Wait for the server to send a response back
    pub async fn await_response(&mut self) -> anyhow::Result<Response> {
        let mut buf = BytesMut::with_capacity(64);
        self.socket.recv_buf(&mut buf).await?;
        match buf.iter().enumerate().find(|(_, b)| **b == b'\n') {
            Some((i, _)) => {
                let response = parse_response_bin(&buf[0..i])?;
                Ok(response)
            }
            None => Err(anyhow!("server did not return a valid response line")),
        }
    }

    /// Send a single request to the configured server and wait for a response back
    pub async fn exchange(&mut self, request: Request) -> anyhow::Result<Response> {
        self.send_request(request).await?;
        let response = self.await_response().await?;
        Ok(response)
    }
}
