use pixelflut;

#[tokio::main]
async fn main() {
    pixelflut::start_server().await;
}
