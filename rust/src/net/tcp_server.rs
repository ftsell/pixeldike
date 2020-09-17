use actix::prelude::*;
use tokio::net::TcpListener;

pub struct TcpServer {
    port: uint,
    listener: TcpListener,
}

impl Actor for TcpServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.listener = TcpListener::bind("0.0.0.0:9876").await.unwrap();
    }
}

impl Default for TcpServer {
    fn default() -> Self {
        TcpServer{
            port: 9876,
        }
    }
}
