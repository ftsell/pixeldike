extern crate tokio;
extern crate futures;
extern crate websocket;

use crate::pixmap::Pixmap;
use crate::servers::PxServer;

use std::net::SocketAddr;
use std::io::{Error, ErrorKind};
use self::tokio::prelude::*;
use self::tokio::io::lines;
use self::tokio::runtime::{TaskExecutor, Runtime};
use self::tokio::reactor::Handle;
use self::tokio::net::TcpStream;
use self::websocket::server::sync::Server;
use self::websocket::server::upgrade::WsUpgrade;
use self::websocket::server::upgrade::sync::Buffer;
use self::websocket::OwnedMessage;
use self::websocket::server::InvalidConnection;
use std::fmt::Debug;
use std::rc::Rc;
use self::websocket::client::r#async::Framed;
use self::websocket::r#async::MessageCodec;


const SUB_PROTO: &str = "pixelflut";


#[derive(Clone)]
pub struct WsServer {
    map: Pixmap
}


impl WsServer {
    pub fn new(map: Pixmap) -> WsServer {
        WsServer {
            map
        }
    }

    fn handle_client(self, socket: Framed<TcpStream, MessageCodec<OwnedMessage>>) {
        let (sink, strm) = socket.split();

        let answer_fut = strm

            // Only take new messages as long as we haven't received a "close"-message
            .take_while(|m| Ok(!m.is_close()))

            // Process message and convert answer to correct type
            .and_then(move |owned_msg: OwnedMessage| {
                Ok(match owned_msg {
                    OwnedMessage::Ping(b) => Some(OwnedMessage::Pong(b)),
                    OwnedMessage::Text(msg) => match self.handle_message(&msg) {
                        Err(e) => Some(OwnedMessage::Text(e.to_string())),
                        Ok(some_answer) => match some_answer {
                            Some(answer) => Some(OwnedMessage::Text(answer)),
                            None => None
                        }
                    }
                    _ => None
                })
            })
            .filter_map(|v| { v })

            // Forward answer to client
            .forward(sink)
            .map_err(|e| { eprintln!("[WS] Error: {}", e) })

            // Send a proper "close"-message once the stream finishes
            .and_then(|(m, sink)| {
                sink.send(OwnedMessage::Close(None));
                Ok(())
            });


        tokio::spawn(answer_fut);
    }
}


impl PxServer for WsServer {
    fn start(self, port: u16) {
        println!("Starting Websocket Server on port {}", port);

        // Bind to socket
        let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
        let listener = Server::bind(addr)
            .expect(format!("[WS]: Could not bind socket on port {}", port).as_str())
            .into_async(&Handle::current()).unwrap();

        // Construct server handling chain
        let server_fut = listener.incoming()

            // We don't want to save streams whose connection drops
            .map_err(|e| {
                eprintln!("[WS] Invalid connection: {}", e.error);
                ()
            })

            // Reject upgrades with incorrect sub-protocol (px)
            .filter_map(|(upgrade, addr)| {
                if !upgrade.protocols().contains(&SUB_PROTO.to_string()) {
                    // Also properly reject the client
                    tokio::spawn(upgrade.reject().then(|_| { Ok(()) }));

                    None
                } else {
                    Some((upgrade, addr))
                }
            })

            // Accept upgrade and setup protocol
            .and_then(|(upgrade, addr)| {
                upgrade.use_protocol(SUB_PROTO).accept()
                    .map_err(|e| { eprintln!("[WS] Error performing protocol upgrade: {}", e) })
                    .map(move |(socket, headers)| { (socket, addr) })
            })

            // Handle PX command
            .for_each(move |(socket, addr)| {
                self.clone().handle_client(socket);
                Ok(())
            });


        tokio::spawn(server_fut);
    }

    fn cmd_get_size(&self) -> Result<Option<String>, String> {
        Ok(Some(self.map.get_size()))
    }

    fn cmd_get_px(&self, x: usize, y: usize) -> Result<Option<String>, String> {
        self.map.get_pixel(x, y)
            .map(|v| { Some(v) })
    }

    fn cmd_set_px(&self, x: usize, y: usize, color: String) -> Result<Option<String>, String> {
        self.map.set_pixel(x, y, color)
            .map(|_| { None })
    }
}
