use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pixelflut::pixmap::{Color, FileBackedPixmap, InMemoryPixmap, Pixmap};
use std::time::Duration;
use tempfile::tempdir;

static COLOR: Color = Color(42, 42, 42);

fn bench_set_pixel(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_pixel");
    group.noise_threshold(0.2);

    group.bench_function("InMemoryPixmap", |b| {
        let pixmap = InMemoryPixmap::default();
        b.iter(|| pixmap.set_pixel(42, 42, black_box(COLOR)))
    });

    group.bench_function("FileBackedPixmap", |b| {
        let dir = tempdir().unwrap();
        let path = dir.path().join("pixmap.pixmap");
        let pixmap = FileBackedPixmap::new(&path, 800, 600, false).unwrap();
        b.iter(|| pixmap.set_pixel(42, 42, black_box(COLOR)))
    });
}

fn bench_get_pixel(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_pixel");

    group.bench_function("InMemoryPixmap", |b| {
        let pixmap = InMemoryPixmap::default();
        pixmap.set_pixel(42, 42, COLOR).unwrap();
        b.iter(|| pixmap.get_pixel(42, 42))
    });

    group.bench_function("FileBackedPixmap", |b| {
        let dir = tempdir().unwrap();
        let path = dir.path().join("pixmap.pixmap");
        let pixmap = FileBackedPixmap::new(&path, 800, 600, false).unwrap();
        pixmap.set_pixel(42, 42, COLOR).unwrap();
        b.iter(|| pixmap.get_pixel(42, 42))
    });
}

fn bench_get_raw_data(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_raw_data");
    group.measurement_time(Duration::from_secs(15));

    group.bench_function("InMemoryPixmap", |b| {
        let pixmap = InMemoryPixmap::default();
        pixmap.put_raw_data(&vec![COLOR; 42]);
        b.iter(|| pixmap.get_raw_data())
    });

    group.bench_function("FileBackedPixmap", |b| {
        let dir = tempdir().unwrap();
        let path = dir.path().join("pixmap.pixmap");
        let pixmap = FileBackedPixmap::new(&path, 800, 600, false).unwrap();
        pixmap.put_raw_data(&vec![COLOR; 42]);
        b.iter(|| pixmap.get_raw_data())
    });
}

fn bench_put_raw_data(c: &mut Criterion) {
    let mut group = c.benchmark_group("put_raw_data");
    group.measurement_time(Duration::from_secs(15));

    group.bench_function("InMemoryPixmap", |b| {
        let pixmap = InMemoryPixmap::default();
        let size = pixmap.get_size().unwrap();
        b.iter(|| pixmap.put_raw_data(black_box(&vec![COLOR; size.0 * size.1])))
    });

    group.bench_function("FileBackedPixmap", |b| {
        let dir = tempdir().unwrap();
        let path = dir.path().join("pixmap.pixmap");
        let pixmap = FileBackedPixmap::new(&path, 800, 600, false).unwrap();
        let size = pixmap.get_size().unwrap();
        b.iter(|| pixmap.put_raw_data(black_box(&vec![COLOR; size.0 * size.1])))
    });
}

criterion_group!(
    benches,
    bench_set_pixel,
    bench_get_pixel,
    bench_put_raw_data,
    bench_get_raw_data
);
criterion_main!(benches);
