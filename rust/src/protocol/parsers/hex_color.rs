use super::Error;
use crate::pixmap::Color;
use anyhow::Result;
use nom::{
    bytes::complete::{tag, take_while_m_n},
    combinator::{map_res, opt},
    sequence::tuple,
    AsChar, Err, IResult,
};

fn is_hex_digit(c: char) -> bool {
    c.is_hex_digit()
}

fn str_to_u8(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn hex_primary(input: &str) -> IResult<&str, u8, ()> {
    map_res(take_while_m_n(2, 2, is_hex_digit), str_to_u8)(input)
}

/// a canvas color encoded through 3 two-character hex digits preceded by an optional '#'
pub(super) fn parse(input: &str) -> IResult<&str, Color, Error> {
    let (input, _) = opt(tag("#"))(input)?;
    let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)
        .map_err(|_| Err::Error(Error::msg(format!("could not parse '{}' as color", input))))?;

    Ok((input, Color(red, green, blue)))
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::TestResult;

    #[test]
    fn parse_color_lowercase() {
        assert_eq!(parse("#ababab").unwrap(), ("", Color(171, 171, 171)))
    }

    #[test]
    fn parse_color_uppercase() {
        assert_eq!(parse("#ABABAB").unwrap(), ("", Color(171, 171, 171)))
    }

    quickcheck! {
        fn with_without_club(use_club: bool) -> TestResult {
            let color = if use_club {
                "#ababab"
            } else {
                "ababab"
            };

            TestResult::from_bool(parse(color).unwrap() == ("", Color(171, 171, 171)))
        }
    }
}
