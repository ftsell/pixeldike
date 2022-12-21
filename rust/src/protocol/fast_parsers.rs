use crate::pixmap::Color;
use crate::protocol::Request;
use anyhow::Error;
use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

fn parse_color(color1: &str, color2: &str, color3: &str) -> Result<Color, Error> {
    Ok(Color(
        u8::from_str_radix(color1, 16)?,
        u8::from_str_radix(color2, 16)?,
        u8::from_str_radix(color3, 16)?,
    ))
}

pub(super) fn parse_px_set(s: &str) -> Result<Request, ()> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"^PX (\d+) (\d+) #?([\dA-F]{2})([\dA-F]{2})([\dA-F]{2})$").unwrap();
    }

    RE.captures(s)
        .and_then(|captures| {
            let x = usize::from_str(&captures[1]).ok()?;
            let y = usize::from_str(&captures[2]).ok()?;
            Some(Request::PxSet(
                x,
                y,
                parse_color(&captures[3], &captures[4], &captures[5]).ok()?,
            ))
        })
        .ok_or(())
}

#[cfg(test)]
#[test]
fn test_fast_parser() {
    assert_eq!(
        parse_px_set("PX 42 43 #890ABC"),
        Ok(Request::PxSet(42, 43, Color(137, 10, 188)))
    );
}
