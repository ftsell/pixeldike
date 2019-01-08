extern crate websocket;

use std::net::*;
use std::thread;
use std::thread::JoinHandle;
use std::time;
use std::sync::{Arc, Mutex};
use self::websocket::server::upgrade::sync::Buffer;
use self::websocket::server::upgrade::WsUpgrade;
use self::websocket::sync::{Server, Client};
use self::websocket::OwnedMessage;

use crate::X_SIZE;
use crate::Y_SIZE;

pub fn start(map: Vec<Vec<Arc<Mutex<String>>>>, port: u16) -> JoinHandle<()> {
    print!("Starting websocket server on port {}...", &port);
    // Bind to port as websocket server
    let address = SocketAddr::new(IpAddr::from(Ipv4Addr::new(0, 0, 0, 0)), port);
    let server = Server::bind(address).unwrap();
    println!("done");

    // Initiate request handling
    thread::spawn(move || {
        for request in server.filter_map(Result::ok) {
            handle_client(map.clone(), request);
        }
    })
}

fn handle_client(map: Vec<Vec<Arc<Mutex<String>>>>,
                 request: WsUpgrade<TcpStream, Option<Buffer>>) {
    thread::spawn(move || {
        if !request.protocols().contains(&"pixelflut-websocket".to_string()) {
            request.reject().unwrap();
            return;
        }
        let mut client = request.use_protocol("pixelflut-websocket").accept().unwrap();

        let ip = client.peer_addr();
        println!("WS: New Client: {:?}", ip);

        // Send size information first
        let msg = format!("SIZE {} {};", X_SIZE, Y_SIZE);
        send_msg(&mut client, msg);

        // Store last update time
        let mut last_update = time::Instant::now() - time::Duration::from_secs(999);

        // Execute the main update-loop
        loop {
            // Sleep between iterations
            thread::sleep(time::Duration::from_millis(10));

            // Send a full update if enough time has elapsed
            if last_update.elapsed().as_secs() >= 5 {
                last_update = time::Instant::now();
                send_full_update(&mut client, &map);
            }

        }
    });
}

fn send_full_update(client: &mut Client<TcpStream>, map: &Vec<Vec<Arc<Mutex<String>>>>) {
    let mut msg = String::new();

    // Iterate over ever pixel in the map and generate a message for it
    for (x, column) in map.iter().enumerate() {
        for (y, row) in column.iter().enumerate() {

            // Capsule for mutex locking
            {
                let entry = row.lock().unwrap();
                msg += format!("PX {} {} {};", x, y, entry).as_mut_str();
            }

            // Send the message if it is above a certain threshold
            if msg.len() > 10000 {
                send_msg(client, msg);
                msg = String::new();
                thread::sleep(time::Duration::from_millis(10));
            }
        }

    }

    // Send rest of message
    send_msg(client, msg);

}

fn send_msg(client: &mut Client<TcpStream>, msg: String) {
    client.send_message(&OwnedMessage::Text(msg))
        .expect("WS: Error sending message");
}
