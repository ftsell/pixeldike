use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use nom::AsBytes;
use pixelflut::net::framing::OldFrame;
use pixelflut::pixmap::{InMemoryPixmap, SharedPixmap};
use pixelflut::state_encoding::SharedMultiEncodings;
use rand::Rng;

fn bench_commands(c: &mut Criterion) {
    const CMD_PX_SET: &[u8] = "PX 0 0 FF00AA\n".as_bytes();
    const CMD_PX_GET: &[u8] = "PX 0 0\n".as_bytes();

    let mut g = c.benchmark_group("command_e2e_benchmark");
    g.noise_threshold(0.05);
    g.confidence_level(0.99);

    g.throughput(Throughput::Bytes(CMD_PX_SET.len() as u64));
    g.bench_function("px_set", |b| {
        let pixmap = SharedPixmap::new(InMemoryPixmap::new(800, 600).unwrap());
        let encodings = SharedMultiEncodings::default();

        b.iter(|| {
            let (frame, _) = OldFrame::from_input(black_box(CMD_PX_SET)).unwrap();
            pixelflut::net::handle_frame(frame, black_box(&pixmap), black_box(&encodings));
        })
    });

    g.throughput(Throughput::Bytes(CMD_PX_GET.len() as u64));
    g.bench_function("px_get", |b| {
        let pixmap = SharedPixmap::new(InMemoryPixmap::new(800, 600).unwrap());
        let encodings = SharedMultiEncodings::default();

        b.iter(|| {
            let (frame, _) = OldFrame::from_input(black_box(CMD_PX_GET)).unwrap();
            pixelflut::net::handle_frame(frame, black_box(&pixmap), black_box(&encodings));
        })
    });

    g.throughput(Throughput::Elements(1));
    g.bench_function("random_command", |b| {
        let mut rng = rand::thread_rng();
        let pixmap = SharedPixmap::new(InMemoryPixmap::new(800, 600).unwrap());
        let encodings = SharedMultiEncodings::default();

        b.iter_batched(
            || {
                let cmd = match rng.gen_range(0..2) {
                    0 => format!(
                        "PX {x} {y} F00AA\n",
                        x = rng.gen_range(0..800),
                        y = rng.gen_range(0..600)
                    ),
                    1 => format!(
                        "PX {x} {y}\n",
                        x = rng.gen_range(0..800),
                        y = rng.gen_range(0..600)
                    ),
                    _ => unreachable!(),
                };
                bytes::Bytes::from(cmd)
            },
            |cmd| {
                let (frame, _) = OldFrame::from_input(cmd).unwrap();
                pixelflut::net::handle_frame(frame, black_box(&pixmap), black_box(&encodings));
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_commands);
criterion_main!(benches);
