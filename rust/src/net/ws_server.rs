//!
//! Server for handling the pixelflut protocol over websocket connections
//!
//! This implementation is currently fairly basic and only really intended to be used by [pixelflut-js](https://github.com/ftsell/pixelflut-js)
//!

use anyhow::Error;
use std::convert::TryInto;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use futures_util::stream::StreamExt;
use sha1::{Digest, Sha1};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::handshake::server::{Callback, ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::tungstenite::Message;

use crate::net::framing::Frame;
use crate::pixmap::traits::{PixmapBase, PixmapRead, PixmapWrite};
use crate::pixmap::SharedPixmap;
use crate::state_encoding::SharedMultiEncodings;

static LOG_TARGET: &str = "pixelflut.net.ws";

/// Options which can be given to [`listen`] for detailed configuration
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WsOptions {
    /// On which address the server should listen
    pub listen_address: SocketAddr,
}

impl Default for WsOptions {
    fn default() -> Self {
        Self {
            listen_address: SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 1234),
        }
    }
}

/// Start the websocket server on a new task.
///
/// This binds to the socket address specified via *options* with TCP but expects only websocket
/// traffic on it. All pixelflut commands must then be passed over the created websocket channel
/// ond not directly via TCP.
///
/// It uses the provided *pixmap* as a pixel data storage and *encodings* for reading cached state
/// command results.
///
/// It returns a JoinHandle to the task that is executing the server logic as well as a Notify
/// instance that can be used to stop the server.
pub fn start_listener<P>(
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
    options: WsOptions,
) -> (JoinHandle<tokio::io::Result<()>>, Arc<Notify>)
where
    P: PixmapBase + PixmapRead + PixmapWrite + Send + Sync + 'static,
{
    let notify = Arc::new(Notify::new());
    let notify2 = notify.clone();
    let handle = tokio::spawn(async move { listen(pixmap, encodings, options, notify2).await });

    (handle, notify)
}

/// Listen on the tpc port defined through *options* while using the given *pixmap* and *encodings*
/// as backing data storage
pub async fn listen<P>(
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
    options: WsOptions,
    notify_stop: Arc<Notify>,
) -> tokio::io::Result<()>
where
    P: PixmapBase + PixmapRead + PixmapWrite + Send + Sync + 'static,
{
    let mut connection_stop_notifies = Vec::new();
    let listener = TcpListener::bind(options.listen_address).await.unwrap();
    info!(
        target: LOG_TARGET,
        "Started websocket listener on {}",
        listener.local_addr().unwrap()
    );

    loop {
        select! {
            res = listener.accept() => {
                let (socket, _) = res?;
                let pixmap = pixmap.clone();
                let encodings = encodings.clone();
                let connection_stop_notify = Arc::new(Notify::new());
                connection_stop_notifies.push(connection_stop_notify.clone());
                tokio::spawn(async move {
                    process_connection(socket, pixmap, encodings, connection_stop_notify).await;
                });
            },
            _ = notify_stop.notified() => {
                log::info!("Stopping ws server on {}", listener.local_addr().unwrap());
                for i_notify in connection_stop_notifies.iter() {
                    i_notify.notify_one();
                }
                break Ok(());
            }
        }
    }
}

async fn process_connection<P>(
    connection: TcpStream,
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
    notify_stop: Arc<Notify>,
) where
    P: PixmapBase + PixmapRead + PixmapWrite,
{
    debug!(
        target: LOG_TARGET,
        "Client connected {}",
        connection.peer_addr().unwrap()
    );
    let websocket = tokio_tungstenite::accept_hdr_async(connection, WebsocketCallback::new())
        .await
        .unwrap();
    let (write, read) = websocket.split();
    let future = read
        .map(|msg| process_received(msg, pixmap.clone(), encodings.clone()))
        .forward(write);

    select! {
        res = future => {
            if let Err(e) = res {
                warn!(target: LOG_TARGET, "Error while handling connection: {}", e)
            }
        },
        _ = notify_stop.notified() => {
            log::info!("Closing connection");
        }
    }
}

fn process_received<P>(
    msg: Result<Message, WsError>,
    pixmap: SharedPixmap<P>,
    encodings: SharedMultiEncodings,
) -> Result<Message, WsError>
where
    P: PixmapBase + PixmapRead + PixmapWrite,
{
    match msg {
        Ok(msg) => match msg {
            Message::Text(msg) => {
                debug!(target: LOG_TARGET, "Received websocket message: {}", msg);

                // TODO improve websocket frame handling
                let frame = Frame::new_from_string(msg);

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

struct WebsocketCallback {}

impl WebsocketCallback {
    const fn new() -> Self {
        Self {}
    }

    fn calc_websocket_accept(&self, websocket_key: &str) -> Result<HeaderValue, Error> {
        // append known string from spec
        let accept_str = websocket_key.to_string() + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

        // hash it
        let mut hasher = Sha1::new();
        hasher.update(accept_str);
        let accept_hash = hasher.finalize();

        // base64 encode and return it
        let accept_b64 = base64::encode(accept_hash);
        HeaderValue::from_str(&accept_b64)
            .map_err(|_| Error::msg("Could not compute Sec-WebSocket-Key header"))
    }
}

impl Callback for WebsocketCallback {
    fn on_request(self, request: &Request, mut response: Response) -> Result<Response, ErrorResponse> {
        // respond with a correct Sec-WebSocket-Key header if it was requested by the client
        if let Some(websocket_key) = request.headers().get("Sec-WebSocket-Key") {
            match websocket_key.to_str() {
                Ok(websocket_key) => match self.calc_websocket_accept(websocket_key) {
                    Ok(websocket_accept) => {
                        response
                            .headers_mut()
                            .insert("Sec-WebSocket-Accept", websocket_accept);
                    }
                    Err(e) => {
                        log::warn!("{}", e);
                    }
                },
                Err(e) => {
                    log::warn!("Received invalid Sec-WebSocket-Key header: {}", e);
                }
            }
        }

        // set the used websocket sub-protocol
        response
            .headers_mut()
            .insert("Sec-WebSocket-Protocol", HeaderValue::from_static("pixelflut"));

        Ok(response)
    }
}
