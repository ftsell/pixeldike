use crate::pixmap::SharedPixmap;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::WebSocketStream;

static LOG_TARGET: &str = "pixelflut.listener.ws";

pub struct WsOptions {
    pub listen_address: SocketAddr,
}

pub async fn listen(pixmap: SharedPixmap, options: WsOptions) {
    let listener = TcpListener::bind(options.listen_address).await.unwrap();
    info!(
        target: LOG_TARGET,
        "Started websocket listener on {}",
        listener.local_addr().unwrap()
    );

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let pixmap = pixmap.clone();
        tokio::spawn(async move {
            process_connection(socket, pixmap);
        });
    }
}

async fn process_connection(mut connection: TcpStream, pixmap: SharedPixmap) {
    debug!(
        target: LOG_TARGET,
        "Client connected {}",
        connection.peer_addr().unwrap()
    );
    //let (ws_write, ws_read) = tokio_tungstenite::accept_async(connection).await.unwrap();
}
