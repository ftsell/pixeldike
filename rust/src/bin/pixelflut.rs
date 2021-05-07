use pixelflut;
use pretty_env_logger;
use std::path::Path;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let path = Path::new("/home/ftsell/pixmap.pixmap");
    let pixmap = pixelflut::pixmap::FileBackedPixmap::new(&path, 800, 600, false).unwrap();
    pixelflut::run_server(pixmap).await;
}
