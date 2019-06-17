use crate::network::protocol::{Command};
use crate::network::px_server::PxServer;
use crate::pixmap::Pixmap;
use futures::stream::Stream;
use std::io::{BufReader, Write};
use std::sync::Arc;
use tokio::io::{lines, AsyncRead, ReadHalf};
use tokio::net::{TcpListener, TcpStream};
use hex::encode;
use std::convert::TryInto;

#[derive(Clone)]
pub struct TcpServer {
    map: Arc<Pixmap>,
}

impl TcpServer {
    pub fn new(map: Arc<Pixmap>) -> TcpServer {
        TcpServer { map }
    }

    pub fn handle_connection(mut self, sock: TcpStream) {
        let (reader, mut writer) = sock.split();
        let reader = BufReader::new(reader);

        // Construct message chain
        let msg_handler = lines(reader)
            // Since all responses are String types, the error needs to be mapped to String as well
            .map_err(|e| e.to_string())
            // Parse command
            .and_then(move |line| Command::parse(&line))
            // Execute command
            .and_then(move |command| self.handle_command(command))
            // Since errors get returned to the user, we pretend they are a correct response
            .or_else(move |e| Ok(e))
            // Write-back answer
            .and_then(move |response| {
                writer.write_all(response.as_bytes())
                    .map_err(|e| eprintln!("[TCP] Could not send answer: {}", e))
            })

            // Sink stream
            .for_each(|()| Ok(()));

        tokio::spawn(msg_handler);
    }
}

impl PxServer for TcpServer {
    fn start(self, listen_address: &String, port: u16) {
        println!("[TCP] Starting Server on {}:{}", listen_address, port);

        // Bind the server socket
        let addr = format!("{}:{}", listen_address, port)
            .parse()
            .expect("[TCP] Could not construct address from listen_address and port");
        let listener = TcpListener::bind(&addr).expect("[TCP] Could not bind server socket");

        // Construct server chain
        let server = listener
            .incoming()
            .map_err(|e| eprintln!("[TCP] Accepting new connection failed: {:?}", e))
            .for_each(move |sock| {
                self.clone().handle_connection(sock);
                Ok(())
            });

        tokio::spawn(server);
    }

    fn get_size(&self) -> String {
        format!("SIZE {} {}", self.map.x_size, self.map.y_size)
    }

    fn get_px(&self, x: usize, y: usize) -> Result<String, String> {
        match self.map.get_pixel(x, y) {
            Err(e) => Err(e),
            Ok(value) => {
                // Split up the color value so that it gets formatted correctly
                let mut colors: Vec<u8> = Vec::new();
                colors.push((((value >> 16) & 0xFF_u32) as u32).try_into().unwrap());
                colors.push((((value >> 8) & 0xFF_u32) as u32).try_into().unwrap());
                colors.push((((value >> 0) & 0xFF_u32) as u32).try_into().unwrap());

                Ok(format!("PX {} {} {}\n", x, y, encode(colors)).to_uppercase())
            }
        }
    }

    fn set_px(&self, x: usize, y: usize, color: u32) -> Result<String, String> {
        self.map.set_pixel(x, y, color)
            .and(self.get_px(x, y))
    }

    fn binary(&self) -> Result<String, String> {
        let snapshot = self.map.snapshot.read().unwrap();
        return Ok(snapshot.clone());
    }
}
