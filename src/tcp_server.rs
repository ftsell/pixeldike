use std::thread::JoinHandle;
use std::sync::Mutex;
use std::sync::Arc;

use crate::X_SIZE;
use crate::Y_SIZE;
use std::thread;

pub fn start(map: Arc<Mutex<Vec<Vec<String>>>>, port: u16) -> JoinHandle<()> {
    print!("Starting TCP PX server...");
    println!("done");

    thread::spawn(|| {

    })
}