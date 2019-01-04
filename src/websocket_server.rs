extern crate websocket;

use std::net::*;
use std::thread;
use std::thread::JoinHandle;
use std::time;
use std::sync::Arc;
use std::sync::Mutex;
use self::websocket::server::upgrade::sync::Buffer;
use self::websocket::server::upgrade::WsUpgrade;
use self::websocket::sync::Server;
use self::websocket::OwnedMessage;

pub fn start(map: Arc<Mutex<Vec<Vec<String>>>>, port: u16) -> JoinHandle<()> {
    print!("Starting websocket server...");
    // Bind to port as websocket server
    let address = SocketAddr::new(IpAddr::from(Ipv4Addr::new(0, 0, 0, 0)), port);
    let server = Server::bind(address).unwrap();
    println!("done");

    // Initiate request handling
    thread::spawn(move || {
        for request in server.filter_map(Result::ok) {
            handle_request(map.clone(), request);
        }
    })
}

fn handle_request(map: Arc<Mutex<Vec<Vec<String>>>>, request: WsUpgrade<TcpStream, Option<Buffer>>) {
    thread::spawn(move || {
        if !request.protocols().contains(&"rust-websocket".to_string()) {
            request.reject().unwrap();
            return;
        }
        let mut client = request.use_protocol("rust-websocket").accept().unwrap();

        let ip = client.peer_addr().unwrap();
        println!("Connection from {}", ip);

        // Execute the main update-loop
        loop {
            let mut msg = String::new();

            // Capsule for map locking
            {
                let map = map.lock().unwrap();

                // Iterate over ever pixel in the map and append it to the msg
                for (x, column) in map.iter().enumerate() {
                    for (y, row) in column.iter().enumerate() {
                        msg += format!("PX {} {} {};", x, y, row).as_mut_str()
                    }
                }
            }

            client.send_message(&OwnedMessage::Text(msg)).unwrap();

            // Wait 100ms until another update is sent
            thread::sleep(time::Duration::from_millis(100));
        }
    });
}
