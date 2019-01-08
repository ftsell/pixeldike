use std::net::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;

use crate::command_handler;
use crate::command_handler::Command;


pub fn start(map: Vec<Vec<Arc<Mutex<String>>>>, port: u16) -> JoinHandle<()> {
    print!("Starting Udp PX server...");
    let socket = setup_udp_socket(port);
    println!("done");

    thread::spawn(move || loop {
        let msg = receive_msg(&socket);

        if msg.is_err() {
            println!("Error: {}", msg.unwrap_err())
        } else {
            let map = map.clone();
            thread::spawn(move || {
                let (_src, msg) = msg.unwrap();
                let cmd = command_handler::parse_message(&msg);

                if cmd.is_err() {
                    let err_msg = cmd.err().unwrap();
                    println!("Error: {}", &err_msg);
                } else {
                    let cmd = cmd.unwrap();

                    let _answer = match cmd {
                        Command::SIZE => command_handler::cmd_size(),
                        Command::PX(x, y, color) => command_handler::cmd_px(&map, x, y, color),
                    };

                    //println!("{}", _answer);
                }
            });
        }
    })
}

fn setup_udp_socket(port: u16) -> UdpSocket {
    let address = SocketAddr::new(IpAddr::from(Ipv4Addr::new(0, 0, 0, 0)), port);

    return UdpSocket::bind(address).expect(&format!("Could not bind to port {}", port));
}

fn receive_msg(socket: &UdpSocket) -> Result<(SocketAddr, String), String> {
    let mut buf = [0; 19];
    let res = socket.recv_from(&mut buf);

    if res.is_err() {
        return Err(res.err().unwrap().to_string());
    }

    let (acm, src) = res.unwrap();

    let msg = String::from_utf8(buf[..acm].to_vec());

    if msg.is_err() {
        return Err(msg.err().unwrap().to_string());
    }

    return Ok((src, msg.unwrap()));
}
