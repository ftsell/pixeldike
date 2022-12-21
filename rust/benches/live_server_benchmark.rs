use criterion::{criterion_group, criterion_main, Criterion};
use pixelflut::net::tcp_server;
use pixelflut::pixmap::{Color, Pixmap};
use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

//
// Run this using:
// perf record -o ~/perf.data --call-graph dwarf --aio -z --sample-cpu cargo bench --all-features 'live_tcp/*'
//

fn bench_tcp(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    rt.spawn(async {
        // arrange pixelflut
        let pixmap = Arc::new(pixelflut::pixmap::InMemoryPixmap::default());
        let encodings = pixelflut::state_encoding::SharedMultiEncodings::default();
        let (server_handle, _) = tcp_server::start_listener(
            pixmap.clone(),
            encodings,
            tcp_server::TcpOptions {
                listen_address: SocketAddr::from_str("127.0.0.1:1234").unwrap(),
            },
        );
        server_handle.await.unwrap().unwrap();
    });

    let mut group = c.benchmark_group("live_tcp");
    group.measurement_time(Duration::from_secs(15));

    group.bench_function("WriteSinglePixel", |b| {
        let tcp = TcpStream::connect("127.0.0.1:1234").unwrap();
        let client = pixelflut::pixmap::RemotePixmap::new(tcp.try_clone().unwrap(), tcp).unwrap();
        b.iter(|| client.set_pixel(0, 0, Color(255, 0, 255)));
    });
}

criterion_group!(benches, bench_tcp);
criterion_main!(benches);
