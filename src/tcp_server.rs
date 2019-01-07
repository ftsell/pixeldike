use std::thread::JoinHandle;
use std::sync::{Mutex, Arc, mpsc};
use std::thread;
use std::net::*;
use std::io::{Read, Write};


use crate::command_handler;
use self::command_handler::Command;


pub fn start(map: Vec<Vec<Arc<Mutex<String>>>>, port: u16) -> JoinHandle<()> {
    print!("Starting TCP PX server...");
    let socket = setup_socket(port);
    println!("done");

    thread::spawn(move || {
        let (tx, rx) = mpsc::channel::<(usize, [u8;19])>();
        let _input_handler = start_input_handler(map, rx);
        loop_server(socket, tx);
    })
}

fn setup_socket(port: u16) -> TcpListener {
    let address = SocketAddr::new(IpAddr::from(Ipv4Addr::new(0, 0, 0, 0)), port);
    TcpListener::bind(address).expect("Could not bind TCP socket")
}

fn start_input_handler(map: Vec<Vec<Arc<Mutex<String>>>>, rx: mpsc::Receiver<(usize, [u8;19])>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            // Receive input from other channels
            let (acm, buf) = rx.recv().expect("All senders to input_handler have closed");

            // Convert input to string
            if let Ok(msg) = String::from_utf8(buf[..acm].to_vec()) {

                // Parse command from string
                if let Ok(cmd) = command_handler::parse_message(msg) {

                    // Execute correct command
                    let answer = match cmd {
                        Command::SIZE => command_handler::cmd_size(),
                        Command::PX(x, y, color) => command_handler::cmd_px(&map, x, y, color)
                    };

                    println!("{}", answer);

                }
            }
        }
    })
}

fn loop_server(socket: TcpListener, tx: mpsc::Sender<(usize, [u8;19])>) {
    loop {
        match socket.accept() {
            Ok((stream, addr)) => {
                handle_client(stream, addr, tx.clone());
            },
            Err(e) => println!("Error: Couldn't get client: {:?}", e)
        }
    }
}

fn handle_client(mut stream: TcpStream, addr: SocketAddr, tx: mpsc::Sender<(usize, [u8;19])>) -> JoinHandle<()> {
    println!("New PX TCP client: {:?}", addr);

    thread::spawn(move || {
        loop {
            if let Ok(msg) = receive_msg(&mut stream) {
                tx.send(msg).expect("Could not send received byte to input_handler");
            } else {
                println!("PX client disconnected: {:?}", addr);
                break;
            }
        }
    })
}

fn receive_msg(stream: &mut TcpStream) -> Result<(usize, [u8;19]), String> {
    // Receive bytes from input stream
    let mut buf = [0; 19];
    let acm = stream.read(&mut buf);
    if let Ok(acm) = acm {

        // If read() returns without having read any bytes, the stream seems to be closed
        if acm == 0 {
            return Err("Stream close".to_string());
        }

        return Ok((acm, buf));

    } else {
        return Err(acm.unwrap_err().to_string());
    }
}

fn send_msg(stream: &mut TcpStream, msg: &String) -> Result<usize, std::io::Error> {
    stream.write(msg.as_bytes())
}
