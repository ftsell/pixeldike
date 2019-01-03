extern crate websocket;

use std::net::*;
use std::thread;
use std::thread::JoinHandle;
use self::websocket::server::upgrade::sync::Buffer;
use self::websocket::server::upgrade::WsUpgrade;
use websocket::sync::Server;
use websocket::OwnedMessage;

pub fn start(port: u16) -> JoinHandle<()> {
    print!("Starting websocket server...");
    // Bind to port as websocket server
    let address = SocketAddr::new(IpAddr::from(Ipv4Addr::new(127, 0, 0, 1)), port);
    let server = Server::bind(address).unwrap();
    println!("done");

    // Initiate request handling
    thread::spawn(move || {
        for request in server.filter_map(Result::ok) {
            handle_request(request);
        }
    })
}

fn handle_request(request: WsUpgrade<TcpStream, Option<Buffer>>) {
    thread::spawn(move || {
        if !request.protocols().contains(&"rust-websocket".to_string()) {
            request.reject().unwrap();
            return;
        }

        let mut client = request.use_protocol("rust-websocket").accept().unwrap();

        let ip = client.peer_addr().unwrap();

        println!("Connection from {}", ip);

        let message = OwnedMessage::Text("Hello".to_string());
        client.send_message(&message).unwrap();

        let (mut receiver, mut sender) = client.split().unwrap();

        for message in receiver.incoming_messages() {
            let message = message.unwrap();

            match message {
                OwnedMessage::Close(_) => {
                    let message = OwnedMessage::Close(None);
                    sender.send_message(&message).unwrap();
                    println!("Client {} disconnected", ip);
                    return;
                }
                OwnedMessage::Ping(ping) => {
                    let message = OwnedMessage::Pong(ping);
                    sender.send_message(&message).unwrap();
                }
                _ => sender.send_message(&message).unwrap(),
            }
        }
    });
}
