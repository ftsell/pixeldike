use std::net::*;
use std::thread::JoinHandle;
use std::thread;
use std::sync::Mutex;
use std::sync::Arc;

use crate::X_SIZE;
use crate::Y_SIZE;


enum Command {
    SIZE,
    PX(usize, usize, String),
}


pub fn start(map: Arc<Mutex<Vec<Vec<String>>>>, port: u16) -> JoinHandle<()> {
    let socket = setup_udp_socket(port);

    thread::spawn(move || {
        loop {
            let msg = receive_msg(&socket);

            if msg.is_err() {
                println!("Error: {}", msg.unwrap_err())
            } else {
                let (src, msg) = msg.unwrap();
                let cmd = parse_message(msg);

                if cmd.is_err() {
                    let err_msg = cmd.err().unwrap();
                    println!("Error: {}", &err_msg);
                    send_msg(&socket, &src, &err_msg);
                } else {
                    let cmd = cmd.unwrap();

                    let answer = match cmd {
                        Command::SIZE => cmd_size(),
                        Command::PX(x, y, color) => cmd_px(&map, x, y, color)
                    };

                    println!("{}", answer);
                    send_msg(&socket, &src, &answer);

                }
            }
        }
    })
}


fn setup_udp_socket(port: u16) -> UdpSocket {
    let address = SocketAddr::new(IpAddr::from(Ipv4Addr::new(127, 0, 0, 1)), port);

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


fn parse_message(msg: String) -> Result<Command, String> {
    if msg.eq(&String::from("SIZE")) {

        return Ok(Command::SIZE);

    } else if msg[..2].eq(&String::from("PX")) {

        // Define iterator over all fields in command and ignore PX part at the beginning
        let mut msg_iterator = msg.split_whitespace();
        msg_iterator.next();
        // Extract values from command
        let x = msg_iterator.next();
        let y = msg_iterator.next();
        let color = msg_iterator.next();

        /*
        let x = msg.get(Range { start: 3, end: 6 });
        let y = msg.get(Range { start: 7, end: 10 });

        // We need to take into account that the transparency value is optional
        let color = msg
            .get(Range { start: 11, end: 19 })
            .or(msg.get(Range { start: 11, end: 17 }));
        */

        // Check that every data point could be extracted
        if !(x.is_some() && y.is_some() && color.is_some()) {
            return Err(String::from("Could not extract data from PX command"));
        }

        let x = x.unwrap().parse::<usize>();
        let y = y.unwrap().parse::<usize>();
        let color = {
            if color.unwrap().len() == 6 {
                (color.unwrap().to_string() + "FF")
            } else {
                color.unwrap().to_string()
            }
        };

        if x.is_err() || y.is_err() {
            return Err(String::from("Could not parse xy position"))
        }

        return Ok(Command::PX(x.unwrap(), y.unwrap(), color));
    }

    return Err(String::from(
        "Could not parse message. It is neither a SIZE nor PX command",
    ));
}


fn send_msg(socket: &UdpSocket, dst: &SocketAddr, msg: &String) {
    let buf = msg.as_bytes();
    socket.send_to(buf, &dst).is_ok();
}


fn cmd_size() -> String{
    format!("SIZE {} {}", X_SIZE, Y_SIZE)
}


fn cmd_px(map: &Arc<Mutex<Vec<Vec<String>>>>, x: usize, y: usize, color: String) -> String {
    let answer = format!("PX {} {} {}", x, y, &color);

    // Check that coordinates are inside the grid
    if x >= X_SIZE || y >= Y_SIZE {
        return format!("Coordinates {}:{} not inside grid (0-{}:0-{})", x, y, X_SIZE-1, Y_SIZE-1);
    }

    // Lock map mutex for modification
    {
        let mut mutex = map.lock().unwrap();
        // Retrieve mutable slices in order to modify the element in place
        let column: &mut Vec<String> = mutex.get_mut(x).unwrap();
        let elem: &mut String = column.get_mut(y).unwrap();

        // Overwrite the contained value of this element
        *elem = color;
    }

    answer
}