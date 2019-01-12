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


pub struct TcpServer {
    map: Pixmap,
    listener: TcpListener,
}


impl TcpServer {
    pub fn new(map: Pixmap, port: u16) -> TcpServer {
        // Bind the server socket
        let addr = format!("0.0.0.0:{}", port).parse().unwrap();
        let listener = TcpListener::bind(&addr)
            .expect(format!("TCP: Could not bind socket on port {}", port).as_str());

        TcpServer {
            map,
            listener,
        }
    }

    fn handle_incoming(arc_self: Arc<Self>, sock: TcpStream) {
        // Split up the reading and writing parts of the socket
        let (reader, mut writer) = sock.split();
        let reader = BufReader::new(reader);

        let lines_handler = lines(reader)
            .for_each(move |line| {
                arc_self.handle_message(&line, &mut writer)
                    .map_err(|e| {
                        let mut msg = e.to_string();
                        msg += "\n";

                        writer.write_all(msg.as_bytes()).unwrap_or_default();
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
    fn start(self) -> () {
        println!("Starting TCP Server");

        let addr = format!("0.0.0.0:{}", 1235).parse().unwrap();
        let listener = TcpListener::bind(&addr)
            .expect(format!("TCP: Could not bind socket on port {}", 1235).as_str());

        let arc_self = Arc::new(self);

        // Pull out a stream of sockets for incoming connections
        let server = listener.incoming()
            .map_err(|e| eprintln!("TCP: Accept new connection failed: {:?}", e))
            .for_each(move |sock| {
                TcpServer::handle_incoming(arc_self.clone(), sock);

                Ok(())
            });

        tokio::spawn(server);
    }

    fn cmd_get_size(&self, answer_channel: &mut Write) -> Result<(), Error> {
        let mut size = self.map.get_size();
        size += "\n";

        answer_channel.write_all(size.as_bytes()).unwrap_or_default();

        Ok(())
    }

    fn cmd_get_px(&self, answer_channel: &mut Write, x: usize, y: usize) -> Result<(), Error> {
        self.map.get_pixel(x, y).and_then(|mut color| {
            color += "\n";
            answer_channel.write_all(color.as_bytes()).unwrap_or_default();

            Ok(())
        });

        Ok(())
    }

    fn cmd_set_px(&self, x: usize, y: usize, color: String) -> Result<(), Error> {
        Ok(self.map.set_pixel(x, y, color).unwrap())
    }
}


/*
fn handle_incoming(sock: TcpStream) -> Spawn {
    // Split up the reading and writing parts of the
    // socket.
    let (reader, writer) = sock.split();

    // ... after which we'll print what happened.
    let handle_conn = bytes_copied.map(|amt| {
        println!("wrote {:?} bytes", amt)
    }).map_err(|err| {
        eprintln!("IO error {:?}", err)
    });

    // Spawn the future as a concurrent task.
    tokio::spawn(handle_conn)
}
*/
