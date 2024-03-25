use crate::net::clients::GenClient;
use crate::net::protocol::{parse_response_bin, Request, Response};
use anyhow::anyhow;
use async_trait::async_trait;
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

#[async_trait]
impl GenClient for UdpClient {
    async fn connect(addr: SocketAddr) -> std::io::Result<Self> {
        let socket = if addr.is_ipv4() {
            UdpSocket::bind(SocketAddr::from_str("0.0.0.0:0").unwrap()).await?
        } else {
            UdpSocket::bind(SocketAddr::from_str("[::]:0").unwrap()).await?
        };
        socket.connect(addr).await?;
        Ok(Self { socket })
    }

    async fn send_request(&mut self, request: Request) -> std::io::Result<()> {
        let mut buf = BytesMut::with_capacity(64).writer();
        request.write(&mut buf).unwrap();
        self.socket.send(&buf.get_ref()).await?;
        Ok(())
    }

    async fn await_response(&mut self) -> anyhow::Result<Response> {
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
}
