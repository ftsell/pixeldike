#[macro_use]
extern crate rustler;

mod atoms {
    rustler::atoms! {}
}

fn add(a: i64, b: i64) -> i64 {
    a + b
}

rustler::init!("PixelflutCanvas.NativeArray", [add]);
