use crate::net::framing::Frame;
use crate::pixmap::{Pixmap, SharedPixmap};
use crate::state_encoding::SharedMultiEncodings;
use bytes::Bytes;
use futures_util::stream::StreamExt;
use std::convert::TryInto;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::tungstenite::Message;

static LOG_TARGET: &str = "pixelflut.net.ws";

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WsOptions {
    pub listen_address: SocketAddr,
}

pub async fn listen<P>(pixmap: SharedPixmap<P>, encodings: SharedMultiEncodings, options: WsOptions)
where
    P: Pixmap + Send + Sync + 'static,
{
    let listener = TcpListener::bind(options.listen_address).await.unwrap();
    info!(
        target: LOG_TARGET,
        "Started websocket listener on {}",
        listener.local_addr().unwrap()
    );

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        tokio::spawn(async move {
            process_connection(socket, pixmap, encodings).await;
        });
    }
}

async fn process_connection<P>(
    connection: TcpStream,
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
) where
    P: Pixmap,
{
    debug!(
        target: LOG_TARGET,
        "Client connected {}",
        connection.peer_addr().unwrap()
    );
    let websocket = tokio_tungstenite::accept_async(connection).await.unwrap();
    let (write, read) = websocket.split();
    read.map(|msg| process_received(msg, pixmap.clone(), encodings.clone()))
        .forward(write)
        .await
        .unwrap();
}

fn process_received<P>(
    msg: Result<Message, WsError>,
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
) -> Result<Message, WsError>
where
    P: Pixmap,
{
    match msg {
        Ok(msg) => match msg {
            Message::Text(msg) => {
                debug!(target: LOG_TARGET, "Received websocket message: {}", msg);

                // TODO improve websocket frame handling
                let (frame, _length) = Frame::from_input(Bytes::from(msg)).unwrap();

                // TODO improve by not sending empty responses
                match super::handle_frame(frame, &pixmap, &encodings) {
                    None => Ok(Message::Text(String::new())),
                    Some(response) => Ok(Message::Text(response.try_into().unwrap())),
                }
            }
            _ => {
                warn!(target: LOG_TARGET, "Could not handle websocket message: {}", msg);
                Ok(Message::text(String::new()))
            }
        },
        Err(e) => {
            warn!(target: LOG_TARGET, "Websocket error: {}", e);
            Ok(Message::Text(String::new()))
        }
    }
}

/*
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
 */
