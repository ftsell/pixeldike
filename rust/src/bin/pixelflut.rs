use pixelflut;
use pretty_env_logger;
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    //let path = Path::new("/home/ftsell/pixmap.pixmap");
    //let pixmap = pixelflut::pixmap::FileBackedPixmap::new(&path, 800, 600, false).unwrap();

    let address = SocketAddr::from_str("148.251.182.214:9876").unwrap();
    let stream = TcpStream::connect(address).unwrap();
    //let pixmap = pixelflut::pixmap::RemotePixmap::new(stream.try_clone().unwrap(), stream).unwrap();
    let pixmap = pixelflut::pixmap::InMemoryPixmap::default();

    pixelflut::run_server(pixmap).await;
}
