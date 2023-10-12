use crate::net::protocol::nom_parsers::ProtocolError;
use crate::pixmap::Color;
use nom::branch::permutation;
use nom::bytes::complete::{tag, take_while_m_n};
use nom::character::is_hex_digit;
use nom::combinator::{map, opt};
use nom::IResult;

fn hex_u8(input: &[u8]) -> u8 {
    input
        .iter()
        .rev()
        .enumerate()
        .map(|(k, &v)| {
            let digit = v as char;
            (digit.to_digit(16).unwrap() << (k * 4)) as u8
        })
        .sum()
}

/// Parse a single u8 from two hex symbols
fn parse_hex_primary(input: &[u8]) -> IResult<&[u8], u8, ProtocolError> {
    map(take_while_m_n(2, 2, is_hex_digit), hex_u8)(input)
}

/// Parse a valid color from the input.
pub(super) fn parse_color(input: &[u8]) -> IResult<&[u8], Color, ProtocolError> {
    let (input, _) = opt(tag("#"))(input)?;
    let (input, (r, g, b)) = permutation((parse_hex_primary, parse_hex_primary, parse_hex_primary))(input)?;
    Ok((input, Color(r, g, b)))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_color_with_tag() {
        let (remainder, color) = parse_color("#FF00AA".as_bytes()).unwrap();
        assert_eq!(remainder.len(), 0);
        assert_eq!(color, Color(0xFF, 0x00, 0xAA));
    }

    #[test]
    fn test_valid_color_without_tag() {
        let (remainder, color) = parse_color("FF00AA".as_bytes()).unwrap();
        assert_eq!(remainder.len(), 0);
        assert_eq!(color, Color(0xFF, 0x00, 0xAA));
    }

    #[test]
    fn test_invalid_color() {
        let res = parse_color("zz".as_bytes());
        assert!(res.is_err())
    }
}
