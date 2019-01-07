extern crate websocket;
extern crate spmc;

use std::net::*;
use std::thread;
use std::thread::JoinHandle;
use std::time;
use std::sync::{Arc, Mutex};
use self::websocket::server::upgrade::sync::Buffer;
use self::websocket::server::upgrade::WsUpgrade;
use self::websocket::sync::Server;
use self::websocket::OwnedMessage;

use crate::X_SIZE;
use crate::Y_SIZE;

pub fn start(map: Vec<Vec<Arc<Mutex<String>>>>, port: u16) -> JoinHandle<()> {
    print!("Starting websocket server...");
    // Bind to port as websocket server
    let address = SocketAddr::new(IpAddr::from(Ipv4Addr::new(0, 0, 0, 0)), port);
    let server = Server::bind(address).unwrap();
    let (tx, rx) = spmc::channel();
    println!("done");

    // Initiate request handling
    let _update_handler = start_update_loop(tx, map.clone());
    thread::spawn(move || {
        for request in server.filter_map(Result::ok) {
            handle_client(rx.clone(), request);
        }
    })
}

fn start_update_loop(tx: spmc::Sender<String>, map: Vec<Vec<Arc<Mutex<String>>>>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            // Sleep between full updates
            thread::sleep(time::Duration::from_millis(500));

            // Construct update message
            let mut msg = String::from(format!("SIZE {} {};", X_SIZE, Y_SIZE));
            // Iterate over ever pixel in the map and append it to the msg
            for (x, column) in map.iter().enumerate() {
                for (y, row) in column.iter().enumerate() {

                    // Capsule for mutex locking
                    {
                        let entry = row.lock().unwrap();
                        msg += format!("PX {} {} {};", x, y, entry).as_mut_str()
                    }
                }
            }

            tx.send(msg);
        }
    })
}

fn handle_client(rx: spmc::Receiver<String>, request: WsUpgrade<TcpStream, Option<Buffer>>) {
    thread::spawn(move || {
        if !request.protocols().contains(&"pixelflut-websocket".to_string()) {
            request.reject().unwrap();
            return;
        }
        let mut client = request.use_protocol("pixelflut-websocket").accept().unwrap();

        let ip = client.peer_addr();
        println!("New websocket client: {:?}", ip);

        // Execute the main update-loop
        loop {
            let msg = rx.recv().expect("Cannot receive message from websocket update loop");
            client.send_message(&OwnedMessage::Text(msg))
                .expect(&format!("Cannot send update to websocket client: {:?}", ip).to_string());
        }
    });
}
