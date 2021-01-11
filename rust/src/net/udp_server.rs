use crate::net::framing::Frame;
use crate::pixmap::SharedPixmap;
use bytes::BytesMut;
use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

static LOG_TARGET: &str = "pixelflut.listener.udp";

pub struct UdpOptions {
    pub listen_address: SocketAddr,
}

pub async fn listen(pixmap: SharedPixmap, options: UdpOptions) {
    let socket = Arc::new(UdpSocket::bind(options.listen_address).await.unwrap());
    info!(
        target: LOG_TARGET,
        "Started udp listener on {}",
        socket.local_addr().unwrap()
    );

    loop {
        let socket = socket.clone();
        let pixmap = pixmap.clone();
        let mut buffer = BytesMut::with_capacity(1024);
        let (num_read, origin) = socket.recv_from(&mut buffer[..]).await.unwrap();

        tokio::spawn(async move {
            process_received(buffer, num_read, origin, socket, pixmap).await;
        });
    }
}

async fn process_received(
    buffer: BytesMut,
    num_read: usize,
    origin: SocketAddr,
    socket: Arc<UdpSocket>,
    pixmap: SharedPixmap,
) {
    let mut buffer = Cursor::new(&buffer[..num_read]);

    let frame = match Frame::check(&mut buffer) {
        Err(_) => return,
        Ok(_) => {
            // reset the cursor so that `parse` can read the same bytes as `check`
            buffer.set_position(0);

            Frame::parse(&mut buffer).ok().unwrap()
        }
    };

    // handle the frame
    let response = super::handle_frame(frame, &pixmap);

    // sen the response back to the client (if there is one)
    match response {
        None => {}
        Some(response) => match socket.send_to(&response.encode()[..], origin).await {
            Err(e) => warn!(
                target: LOG_TARGET,
                "Could not send response to {} because: {}", origin, e
            ),
            Ok(_) => {}
        },
    };
}
