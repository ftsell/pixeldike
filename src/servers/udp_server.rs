extern crate futures;
extern crate tokio;

use crate::pixmap::Pixmap;
use crate::servers::PxServer;

use self::tokio::codec::LinesCodec;
use self::tokio::net::{UdpFramed, UdpSocket};
use self::tokio::prelude::*;

use std::ops::RangeInclusive;

#[derive(Clone)]
pub struct UdpServer {
    map: Pixmap,
}

impl UdpServer {
    pub fn new(map: Pixmap) -> UdpServer {
        UdpServer { map }
    }
}

impl PxServer for UdpServer {
    fn start(self, port: u16) {
        println!("Starting UDP Server on port {}", port);

        // Bind the server socket
        let addr = format!("0.0.0.0:{}", port).parse().unwrap();
        let socket = UdpSocket::bind(&addr)
            .expect(format!("[UDP]: Could not bind socket on port {}", port).as_str());

        // Create framed socket for easier interaction
        let codec = LinesCodec::new();
        let framed_socket = UdpFramed::new(socket, codec);
        let (_sender, receiver) = framed_socket.split();

        let server_fut = receiver
            .map_err(|e| eprintln!("[UDP] Could not receive datagram: {}", e))
            .then(move |res| {
                let (line, addr) = res.unwrap();
                self.handle_message(&line)
                    .map(|some_answer| some_answer.map(|answer| (answer, addr)))
                    .map_err(|err| (err, addr))
            })
            .or_else(|(e, addr)| {
                eprintln!("{}", e.to_string());
                Ok(Some((e.to_string(), addr)))
            })
            .filter_map(|some_answer| some_answer)
            .for_each(move |(_answer, _addr)| {
                // println!("{}", answer);
                // println!("{}", addr);

                // TODO actually send answer

                Ok(())
            });

        tokio::spawn(server_fut);
    }

    fn cmd_get_size(&self) -> Result<Option<String>, String> {
        Ok(Some(self.map.get_size()))
    }

    fn cmd_get_px(&self, x: usize, y: usize) -> Result<Option<String>, String> {
        self.map.get_pixel(x, y).map(Some)
    }

    fn cmd_set_px(&self, x: usize, y: usize, color: String) -> Result<Option<String>, String> {
        self.map.set_pixel(x, y, color).map(|_| None)
    }

    fn cmd_get_state(
        &self,
        x: RangeInclusive<usize>,
        y: RangeInclusive<usize>,
    ) -> Result<Option<String>, String> {
        self.map.get_state(x, y).map(Some)
    }
}
