use pixelflut;
use pretty_env_logger;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    pixelflut::start_server().await;
}
