use std::{simd::prelude::*, time};

use core::arch::x86_64::__m256i;

use crate::fast::{parse_hex_trick, parse_int_trick};

pub fn read_request_lines_simd(buf: &[u8], mut setpx: impl FnMut(u16, u16, u32)) {
    use std::simd::prelude::*;
    let mut pos = 0;
    while pos < buf.len() - 32 {
        let chunk: Simd<u8, 32> = Simd::from_slice(&buf[pos..pos + 32]);
        //print_simd_str(chunk);
        let nl = simd_first_newline(chunk).unwrap();
        //println!("nl: {nl}");
        pos += nl + 1;
        let mask: Mask<i8, 32> = Mask::from_bitmask((u32::MAX << nl) as u64);
        let line = mask.select(Simd::splat(0), chunk);
        //print_simd_str(line);
        parse_simd_line(line, &mut setpx);
    }
}

pub fn simd_count_newlines(buf: &[u8]) -> usize {
    let mut pos = 0;
    let mut nl: usize = 0;
    while pos < buf.len() - 32 {
        let chunk: Simd<u8, 32> = Simd::from_slice(&buf[pos..pos + 32]);
        let nls = chunk.simd_eq(Simd::splat('\n' as u8)).to_bitmask();
        nl += nls.count_ones() as usize;
        pos += 32;
    }
    nl
}

pub fn read_request_lines_simd_staged(
    buf: &[u8],
    intermediate: &mut Vec<u8>,
    setpx: impl FnMut(u16, u16, u32),
) {
    let t0 = time::Instant::now();
    let requests = simd_count_newlines(buf);
    let tbuf = time::Instant::now();
    intermediate.resize(requests * 32, 0);
    let t1 = time::Instant::now();
    align_requests(buf, intermediate.as_mut());
    let t2 = time::Instant::now();
    handle_aligned_requests(intermediate.as_slice(), setpx);
    let t3 = time::Instant::now();
    println!(
        "count: {} alloc: {} align: {} handle: {}",
        (tbuf - t0).as_secs_f64(),
        (t1 - tbuf).as_secs_f64(),
        (t2 - t1).as_secs_f64(),
        (t3 - t2).as_secs_f64()
    );
}

#[inline(always)]
pub fn align_requests(buf: &[u8], out: &mut [u8]) {
    let mut inpos = 0;
    let mut outpos = 0;
    while inpos < buf.len() - 32 {
        let chunk: Simd<u8, 32> = Simd::from_slice(&buf[inpos..inpos + 32]);
        let nl = simd_first_newline(chunk).unwrap();
        inpos += nl + 1;
        let mask: Mask<i8, 32> = Mask::from_bitmask((u32::MAX << nl) as u64);
        let line = mask.select(Simd::splat(0), chunk);
        let aligned = align_simd_req_line(line);
        unsafe {
            out.get_unchecked_mut(outpos..outpos + 32)
                .copy_from_slice(aligned.as_array().as_slice());
        }
        outpos += 32;
    }
}

#[inline(always)]
pub fn handle_aligned_requests(buf: &[u8], mut setpx: impl FnMut(u16, u16, u32)) {
    for chunk in buf.chunks_exact(32) {
        let x = parse_int_trick(u64::from_be_bytes(chunk[08..16].try_into().unwrap()));
        let y = parse_int_trick(u64::from_be_bytes(chunk[16..24].try_into().unwrap()));
        let col = parse_hex_trick(u64::from_be_bytes(chunk[24..32].try_into().unwrap()));

        setpx(x as u16, y as u16, col);
    }
}

const IDX_32: [u8; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
    29, 30, 31,
];

const IDX_64: [u8; 64] = [
    00, 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51,
    52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,
];

#[inline(always)]
fn align_simd_req_line(mut line: Simd<u8, 32>) -> Simd<u8, 32> {
    use std::simd::prelude::*;
    let mask = line.simd_gt(Simd::splat(' ' as u8));
    let tok_mask = mask.to_bitmask();
    line = mask.select(line, Simd::splat(0));
    const OOB: usize = 31;
    //println!("{tok_mask:b}");
    let aligned = match tok_mask {
        // HELP / SIZE
        0b1111 => line,
        #[rustfmt::skip]
        0b1111110101011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB, OOB, OOB,   3
            , OOB, OOB, OOB, OOB, OOB, OOB, OOB,   5
            , OOB, OOB,   7,   8,   9,  10,  11,  12
            ]),

        #[rustfmt::skip]
        // PX XX Y CCCCCC
        0b11111101011011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB, OOB,   3,   4
            , OOB, OOB, OOB, OOB, OOB, OOB, OOB,   6
            , OOB, OOB,   8,   9,  10,  11,  12,  13
            ]),
        #[rustfmt::skip]
        // PX XXX Y CCCCCC
        0b111111010111011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB,   3,   4,   5
            , OOB, OOB, OOB, OOB, OOB, OOB, OOB,   7
            , OOB, OOB,   9,  10,  11,  12,  13,  14
            ]),
        #[rustfmt::skip]
        // PX XXXX Y CCCCCC
        0b1111110101111011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB,   3,   4,   5,   6
            , OOB, OOB, OOB, OOB, OOB, OOB, OOB,   8
            , OOB, OOB,  10,  11,  12,  13,  14,  15
            ]),
        #[rustfmt::skip]
        // PX X YY CCCCCC
        0b11111101101011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB, OOB, OOB,   3
            , OOB, OOB, OOB, OOB, OOB, OOB,   5,   6
            , OOB, OOB,   8,   9,  10,  11,  12,  13
            ]),
        #[rustfmt::skip]
        // PX XX YY CCCCCC
        0b111111011011011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB, OOB,   3,   4
            , OOB, OOB, OOB, OOB, OOB, OOB,   6,   7
            , OOB, OOB,   9,  10,  11,  12,  13,  14
            ]),
        #[rustfmt::skip]
        // PX XXX YY CCCCCC
        0b1111110110111011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB,   3,   4,   5
            , OOB, OOB, OOB, OOB, OOB, OOB,   7,   8
            , OOB, OOB,  10,  11,  12,  13,  14,  15
            ]),
        #[rustfmt::skip]
        // PX XXXX YY CCCCCC
        0b11111101101111011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB,   3,   4,   5,   6
            , OOB, OOB, OOB, OOB, OOB, OOB,   8,   9
            , OOB, OOB,  11,  12,  13,  14,  15,  16
            ]),
        #[rustfmt::skip]
        // PX X YYY CCCCCC
        0b111111011101011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB, OOB, OOB,   3
            , OOB, OOB, OOB, OOB, OOB,   5,   6,   7
            , OOB, OOB,   9,  10,  11,  12,  13,  14
            ]),
        #[rustfmt::skip]
        // PX XX YYY CCCCCC
        0b1111110111011011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB, OOB,   3,   4
            , OOB, OOB, OOB, OOB, OOB,   6,   7,   8
            , OOB, OOB,  10,  11,  12,  13,  14,  15
            ]),
        #[rustfmt::skip]
        // PX XXX YYY CCCCCC
        0b11111101110111011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB,   3,   4,   5
            , OOB, OOB, OOB, OOB, OOB,   7,   8,   9
            , OOB, OOB,  11,  12,  13,  14,  15,  16
            ]),
        #[rustfmt::skip]
        // PX XXXX YYY CCCCCC
        0b111111011101111011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB,   3,   4,   5,   6
            , OOB, OOB, OOB, OOB, OOB,   8,   9,  10
            , OOB, OOB,  12,  13,  14,  15,  16,  17
            ]),
        #[rustfmt::skip]
        // PX X YYYY CCCCCC
        0b1111110111101011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB, OOB, OOB,   3
            , OOB, OOB, OOB, OOB,   5,   6,   7,   8
            , OOB, OOB,  10,  11,  12,  13,  14,  15
            ]),
        #[rustfmt::skip]
        // PX XX YYYY CCCCCC
        0b11111101111011011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB, OOB,   3,   4
            , OOB, OOB, OOB, OOB,   6,   7,   8,   9
            , OOB, OOB,  11,  12,  13,  14,  15,  16
            ]),
        #[rustfmt::skip]
        // PX XXX YYYY CCCCCC
        0b111111011110111011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB, OOB,   3,   4,   5
            , OOB, OOB, OOB, OOB,   7,   8,   9,  10
            , OOB, OOB,  12,  13,  14,  15,  16,  17
            ]),
        #[rustfmt::skip]
        // PX XXXX YYYY CCCCCC
        0b1111110111101111011 => simd_swizzle!(line,
            [   0,   1, OOB, OOB, OOB, OOB, OOB, OOB
            , OOB, OOB, OOB, OOB,   3,   4,   5,   6
            , OOB, OOB, OOB, OOB,   8,   9,  10,  11
            , OOB, OOB,  13,  14,  15,  16,  17,  18
            ]),

        _ => panic!(),
    };
    aligned
}

#[inline(always)]
fn parse_simd_line(line: Simd<u8, 32>, mut setpx: impl FnMut(u16, u16, u32)) {
    //print_simd_str(line);
    use std::simd::prelude::*;
    let aligned = align_simd_req_line(line);
    let raw: __m256i = aligned.into();
    let u64s: Simd<u64, 4> = raw.into();
    let x = parse_int_trick(u64::from_be(u64s[1]));
    let y = parse_int_trick(u64::from_be(u64s[2]));
    let col = parse_hex_trick(u64::from_be(u64s[3]));

    setpx(x as u16, y as u16, col);
}

#[inline(always)]
pub fn simd_first_newline(simd: Simd<u8, 32>) -> Option<usize> {
    use std::simd::prelude::*;
    let newline: Mask<_, 32> = simd.simd_eq(Simd::splat('\n' as u8));
    newline.first_set()
}

pub fn parse_simd() {
    let mut buf: [u8; 32] = [' ' as u8; 32];
    let input = "   PX 123   456 ABCDE\n";
    buf[0..input.len()].copy_from_slice(input.as_bytes());
    let simd: Simd<u8, 32> = Simd::from_array(buf);
    let spaces: Simd<i8, 32> = simd.simd_eq(Simd::splat(' ' as u8)).to_int();
    let left_spaces = spaces.rotate_elements_left::<1>();
    let end = (left_spaces - spaces).simd_eq(Simd::splat(-1));
    let start = (left_spaces - spaces)
        .rotate_elements_right::<1>()
        .simd_eq(Simd::splat(1));

    println!("{:3?}", simd);
    println!("{:3?}", start.to_int());
    println!("{:3?}", end.to_int());
}

const IDX: [u8; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
    29, 30, 31,
];
const SIMD_IDX: Simd<u8, 32> = Simd::from_array(IDX);

pub fn print_simd_str(simd: Simd<u8, 32>) {
    let array = simd.to_array();
    let str = std::str::from_utf8(&array).unwrap();
    println!("{:?}", str);
}

#[cfg(test)]
mod tests {
    extern crate test;

    use test::Bencher;

    use crate::Pixmap;

    use super::*;

    #[test]
    fn test_align_simd_req_line() {
        let mut buf: [u8; 32] = [' ' as u8; 32];
        let input = "PX 1 4 ABCDEF\n";
        buf[0..input.len()].copy_from_slice(input.as_bytes());
        let simd: Simd<u8, 32> = Simd::from_array(buf);
        let aligned = align_simd_req_line(simd);
        println!("{aligned:?}");
    }

    #[test]
    fn test_parse_simd_line() {
        for x in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 23, 543, 1234] {
            for y in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 23, 543, 1234] {
                for col in [0xabcdef, 0x000000, 0x123456, 0xdedbef] {
                    let mut buf: [u8; 32] = [' ' as u8; 32];
                    let input = format!("PX {x} {y} {col:06x}\n");
                    //println!("{input:?}");

                    buf[0..input.len()].copy_from_slice(input.as_bytes());
                    let simd: Simd<u8, 32> = Simd::from_array(buf);

                    //let aligned = align_simd_req_line(simd);
                    //println!("{aligned:?}");
                    let mut px = 9999;
                    let mut py = 9999;
                    let mut pcol = 0x9999;
                    parse_simd_line(simd, |x, y, c| {
                        px = x;
                        py = y;
                        pcol = c;
                    });
                    assert_eq!(x, px, "x doesn't match");
                    assert_eq!(y, py, "y doesn't match");
                    assert_eq!(col, pcol, "col doesn't match {col:x} {pcol:x}");
                }
            }
        }
    }

    #[bench]
    fn bench_align_simd_req_line(b: &mut Bencher) {
        let mut buf: [u8; 32] = [' ' as u8; 32];
        let input = "PX 1 4 ABCDEF\n";
        buf[0..input.len()].copy_from_slice(input.as_bytes());
        let simd: Simd<u8, 32> = Simd::from_array(buf);
        b.iter(|| align_simd_req_line(std::hint::black_box(simd)));
    }

    #[bench]
    fn bench_parse_req_line(b: &mut Bencher) {
        let mut buf: [u8; 32] = [' ' as u8; 32];
        let input = "PX 1 4 ABCDEF\n";
        buf[0..input.len()].copy_from_slice(input.as_bytes());
        let simd: Simd<u8, 32> = Simd::from_array(buf);
        let mut sum = 0;
        b.iter(|| {
            parse_simd_line(std::hint::black_box(simd), |a, b, c| {
                sum += a as usize;
                sum += b as usize;
                sum += c as usize;
            })
        });
        println!("{sum}");
    }

    #[test]
    fn test_read_request_line_simd() {
        let input = std::fs::read("testcase.txt").expect("no testcase file found");
        let buf = input.as_slice();
        let mut pixmap = Pixmap::new(1920, 1080);
        read_request_lines_simd(std::hint::black_box(buf), |x, y, c| {
            let idx = y as usize * pixmap.width as usize + x as usize;
            if let Some(px) = pixmap.pixels.get_mut(idx) {
                *px = c;
            }
        })
    }

    #[test]
    fn test_read_request_line_simd_staged() {
        let input = std::fs::read("testcase.txt").expect("no testcase file found");
        let buf = input.as_slice();
        let mut intemediate = Vec::new();
        let mut pixmap = Pixmap::new(1920, 1080);
        read_request_lines_simd_staged(std::hint::black_box(buf), &mut intemediate, |x, y, c| {
            let idx = y as usize * pixmap.width as usize + x as usize;
            if let Some(px) = pixmap.pixels.get_mut(idx) {
                *px = c;
            }
        });
        println!("sum: {}", pixmap.pixels.iter().copied().sum::<u32>());
    }
}
