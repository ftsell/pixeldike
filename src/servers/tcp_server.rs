extern crate tokio;
extern crate futures;

use crate::pixmap::Pixmap;
use crate::servers::PxServer;

use self::tokio::prelude::*;
use self::tokio::io::{lines, write_all};
use self::tokio::net::{TcpListener, TcpStream};
use self::futures::lazy;

use std::io::{Read, Write, BufReader, BufRead, Error};
use std::sync::Arc;


#[derive(Clone)]
pub struct TcpServer {
    map: Pixmap
}


impl TcpServer {
    pub fn new(map: Pixmap) -> TcpServer {
        TcpServer {
            map
        }
    }

    fn handle_incoming(self, sock: TcpStream) {
        // Split up the reading and writing parts of the socket
        let (reader, mut writer) = sock.split();
        let reader = BufReader::new(reader);

        let lines_handler = lines(reader)
            .for_each(move |line| {
                self.handle_message(&line)
                    .map_err(|e| {
                        let mut msg = e.to_string();
                        msg += "\n";

                        writer.write_all(msg.as_bytes()).unwrap_or_default();
                    })
                    .map(|answer| {
                        match answer {
                            Some(mut v) => {
                                v += "\n";
                                writer.write_all(v.as_bytes()).unwrap_or_default();
                                ()
                            }
                            _ => ()
                        }
                    });

                Ok(())
            })
            .map_err(|e| {
                eprintln!("TCP: Could not handle client: {:?}", e);
            });

        tokio::spawn(lines_handler);
    }
}

impl PxServer for TcpServer {
    fn start(self, port: u16) {
        println!("Starting TCP Server");

        // Bind the server socket
        let addr = format!("0.0.0.0:{}", port).parse().unwrap();
        let listener = TcpListener::bind(&addr)
            .expect(format!("TCP: Could not bind socket on port {}", 1235).as_str());

        // Pull out a stream of sockets for incoming connections
        let server = listener.incoming()
            .map_err(|e| eprintln!("TCP: Accept new connection failed: {:?}", e))
            .for_each(move |sock| {
                TcpServer::handle_incoming(self.clone(), sock);
                Ok(())
            });

        tokio::spawn(server);
    }

    fn cmd_get_size(&self) -> Result<Option<String>, String> {
        Ok(Some(self.map.get_size()))
    }

    fn cmd_get_px(&self, x: usize, y: usize) -> Result<Option<String>, String> {
        self.map.get_pixel(x, y)
            .map(|v| {Some(v)})
    }

    fn cmd_set_px(&self, x: usize, y: usize, color: String) -> Result<Option<String>, String> {
        self.map.set_pixel(x, y, color)
            .map(|_| {None})
    }
}
