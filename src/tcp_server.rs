use std::thread::JoinHandle;
use std::sync::{Mutex, Arc};
use std::thread;
use std::net::*;
use std::io::{BufReader, BufRead};


use crate::command_handler;
use self::command_handler::Command;


pub fn start(map: Vec<Vec<Arc<Mutex<String>>>>, port: u16) -> JoinHandle<()> {
    print!("Starting TCP PX server on port {}...", &port);
    let socket = setup_socket(port);
    println!("done");

    thread::spawn(move || {
        loop_server(socket, map);
    })
}

fn setup_socket(port: u16) -> TcpListener {
    let address = SocketAddr::new(IpAddr::from(Ipv4Addr::new(0, 0, 0, 0)), port);
    TcpListener::bind(address).expect("Could not bind TCP socket")
}

fn loop_server(socket: TcpListener, map: Vec<Vec<Arc<Mutex<String>>>>) {
    loop {
        match socket.accept() {
            Ok((stream, addr)) => {
                handle_client(stream, addr, map.clone());
            }
            Err(e) => println!("Error: Couldn't get client: {:?}", e)
        }
    }
}

fn handle_client(stream: TcpStream, addr: SocketAddr, map: Vec<Vec<Arc<Mutex<String>>>>) -> JoinHandle<()> {
    println!("New PX TCP client: {:?}", addr);
    let mut reader = BufReader::new(stream);

    thread::spawn(move || {
        loop {
            // Receive message from stream
            match receive_msg(&mut reader) {
                Ok(buf) =>

                    // Decode it from UTF-8
                    match String::from_utf8(buf) {
                        Ok(msg) =>

                            // Parse command
                            match command_handler::parse_message(&msg) {
                                Ok(cmd) => {

                                    // Execute the correct command
                                    let _answer = match cmd {
                                        Command::SIZE => command_handler::cmd_size(),
                                        Command::PX(x, y, color) => command_handler::cmd_px(&map, x, y ,color)
                                    };

                                },

                                Err(e) => println!("PX: Could not parse command '{}': {}", msg, e)
                            },

                        Err(e) => println!("PX: Could not decode UTF-8 message from client {:?}: {}", addr, e)
                    },

                Err(e) => {
                    println!("Error receiving from PX client {:?}: {}", addr, e);
                    break;
                }
            }
        }
    })
}

fn receive_msg(reader: &mut BufReader<TcpStream>) -> Result<Vec<u8>, String> {
    // Receive bytes from input stream
    let mut buf = Vec::new();
    let acm = reader.read_until(";".as_bytes()[0], &mut buf);
    if let Ok(acm) = acm {

        // If read() returns without having read any bytes, the stream seems to be closed
        if acm == 0 {
            return Err("Received 0 bytes from peer".to_string());
        }

        return Ok(buf);
    } else {
        return Err(acm.unwrap_err().to_string());
    }
}
