extern crate futures;
extern crate tokio;

use crate::pixmap::Pixmap;
use crate::servers::PxServer;

use self::tokio::io::lines;
use self::tokio::net::{TcpListener, TcpStream};
use self::tokio::prelude::*;

use std::io::BufReader;
use std::ops::RangeInclusive;

#[derive(Clone)]
pub struct TcpServer {
    map: Pixmap,
}

impl TcpServer {
    pub fn new(map: Pixmap) -> TcpServer {
        TcpServer { map }
    }

    fn handle_incoming(self, incoming: TcpStream) {
        // Split up the reading and writing parts of the socket
        let (reader, mut writer) = incoming.split();
        let reader = BufReader::new(reader);

        // Construct message chain
        let fut = lines(reader)
            .and_then(move |line| -> std::io::Result<Option<String>> { self.handle_message(&line) })
            .or_else(|e| -> Result<Option<String>, ()> {
                eprintln!("{}", e.to_string());
                Ok(Some(e.to_string()))
            })
            .filter_map(|some_answer| some_answer)
            .for_each(move |mut answer| {
                answer += "\n";
                writer
                    .write_all(answer.as_bytes())
                    .map_err(|e| eprintln!("[TCP] Could not send answer: {}", e))
                    .unwrap();

                Ok(())
            });

        // Schedule message handling for this client
        tokio::spawn(fut);
    }
}

impl PxServer for TcpServer {
    fn start(self, port: u16) {
        println!("Starting TCP Server on port {}", port);

        // Bind the server socket
        let addr = format!("0.0.0.0:{}", port).parse().unwrap();
        let listener = TcpListener::bind(&addr)
            .expect(format!("[TCP]: Could not bind socket on port {}", port).as_str());

        // Pull out a stream of sockets for incoming connections
        let server = listener
            .incoming()
            .map_err(|e| eprintln!("[TCP]: Accept new connection failed: {:?}", e))
            .for_each(move |sock| {
                self.clone().handle_incoming(sock);
                Ok(())
            });

        tokio::spawn(server);
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
