use crate::pixmap::Pixmap;

use std::thread::{JoinHandle, spawn, sleep};
use std::net::TcpStream;
use std::net::SocketAddr;
use core::time::Duration;
use std::cmp;
use std::io::Write;
use std::io::BufReader;
use std::io::BufRead;


pub fn start(mut pixmap: Pixmap, addr: SocketAddr) -> JoinHandle<()> {
    let mut strm = TcpStream::connect(addr)
        .expect("Could not connect to pixelflut server");
    println!("Connected to remote pixelflut server");

    let size_answer = send_and_receive(&mut strm, "SIZE\n".to_string());
    parse_size_answer(&mut pixmap, size_answer);

    let mut ix = 0;
    let mut iy = 0;

    spawn(move || {
        const SIZE: usize = 150;

        loop {
            if ix >= pixmap.x_size {
                ix = 0;
                iy = iy + SIZE;
            }

            if iy >= pixmap.y_size {
                ix = 0;
                iy = 0;
            }

            let x_start = ix;
            let x_end = cmp::min(ix + SIZE, pixmap.x_size) -1;
            let y_start = iy;
            let y_end = cmp::min(iy + SIZE, pixmap.y_size) -1;

            let cmd = format!("STATE {} {} {} {}", x_start, x_end, y_start, y_end);
            let state_answer = send_and_receive(&mut strm, cmd);
            parse_state_answer(&pixmap, state_answer);
            
            // Increase counters for next iteration
            ix = x_end + 1;
            iy = y_start;

            // Sleep before next iteration because too much load is bad
            sleep(Duration::from_millis(100));
        }
    })
}

fn send_and_receive(strm: &mut TcpStream, msg: String) -> String {
    strm.write_all(msg.as_bytes())
        .expect("Connection to server interrupted");

    let mut reader = BufReader::new(strm);
    let mut answer = String::new();
    reader.read_line(&mut answer)
        .expect("Connection to server interrupted");

    return answer;
}

fn parse_size_answer(pixmap: &mut Pixmap, msg: String) {
    let msg = msg.replace("\n", "");
    let split: Vec<&str> = msg.split(" ").collect();

    let x = split.get(1)
        .expect("Could not parse answer from server for SIZE command")
        .parse().unwrap();
    let y = split.get(2)
        .expect("Could not parse answer from server for SIZE command")
        .parse().unwrap();

    pixmap.set_size(x, y);

    println!("Setup canvas size as {}x{}", pixmap.x_size, pixmap.y_size);
}

fn parse_state_answer(pixmap: &Pixmap, msg: String) {
    // Variables which save the range of received pixels
    let mut x_start: usize = 0;
    let mut x_end: usize = 0;
    let mut y_start: usize = 0;
    let mut y_end: usize = 0;

    // Iterator variables needed to know which pixel is currently processed
    let mut ix: usize = 0;
    let mut iy: usize = 0;

    // Iterate over the message to split it by "," symbols
    let mut builder = String::new();
    for i in msg.chars() {
        if i == ',' {

            if builder.starts_with("STATE") {
                // With this information we can setup the x and y ranges
                let split: Vec<&str> = builder.split(" ").collect();

                x_start = split[1].parse().unwrap();
                x_end = split[2].parse().unwrap();
                y_start = split[3].parse().unwrap();
                y_end = split[4].parse().unwrap();

                ix = x_start;
                iy = y_start;
            }

            else {
                // Otherwise it is pixel data
                pixmap.set_pixel(ix, iy, builder).unwrap();
                builder = String::new();

                if iy == y_end {
                    iy = y_start;
                    ix += 1;
                } else {
                    iy += 1;
                }
            }

        }

        else {
            builder += i.to_string().as_str();
        }
    }
}
