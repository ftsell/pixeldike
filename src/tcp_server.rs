use std::thread::JoinHandle;
use std::sync::Mutex;
use std::sync::Arc;
use std::thread;
use std::net::*;
use std::io::{Read, Write};


use crate::command_handler;
use self::command_handler::Command;


pub fn start(map: Arc<Mutex<Vec<Vec<String>>>>, port: u16) -> JoinHandle<()> {
    print!("Starting TCP PX server...");
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

fn loop_server(socket: TcpListener, map: Arc<Mutex<Vec<Vec<String>>>>) {
    loop {
        match socket.accept() {
            Ok((stream, addr)) => {
                handle_client(stream, addr, map.clone());
            },
            Err(e) => println!("Error: Couldn't get client: {:?}", e)
        }
    }
}

fn handle_client(mut stream: TcpStream, addr: SocketAddr, map: Arc<Mutex<Vec<Vec<String>>>>) -> JoinHandle<()> {
    println!("New PX TCP client: {:?}", addr);

    thread::spawn(move || {
        loop {
            if let Ok(msg) = receive_msg(&mut stream) {
                if let Ok(cmd) = command_handler::parse_message(msg) {
                    let answer = match cmd {
                        Command::SIZE => command_handler::cmd_size(),
                        Command::PX(x, y, color) => command_handler::cmd_px(&map, x, y, color)
                    };

                    send_msg(&mut stream, &answer).unwrap();

                }
            }
        }
    })
}

fn receive_msg(stream: &mut TcpStream) -> Result<String, String> {
    // Receive bytes from input stream
    let mut buf = [0; 19];
    let received = stream.read(&mut buf);
    if let Ok(acm) = received {

        // Decode input bytes as UTF-8
        let msg = String::from_utf8(buf[..acm].to_vec());
        if let Ok(msg) = msg {

            return Ok(msg);

        } else {
            return Err(msg.unwrap_err().to_string());
        }

    } else {
        return Err(received.unwrap_err().to_string());
    }
}

fn send_msg(stream: &mut TcpStream, msg: &String) -> Result<usize, std::io::Error> {
    stream.write(msg.as_bytes())
}
