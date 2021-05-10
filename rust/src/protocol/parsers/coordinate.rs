use super::Error;
use nom::bytes::complete::take_while1;
use nom::combinator::map_res;
use nom::IResult;
use nom::{AsChar, Err};
use std::num::ParseIntError;
use std::str::FromStr;

fn is_digit(c: char) -> bool {
    c.is_dec_digit()
}

fn str_to_usize(input: &str) -> Result<usize, ParseIntError> {
    usize::from_str(input)
}

/// a canvas coordinate represented as a decimal digit
pub(super) fn parse(input: &str) -> IResult<&str, usize, Error> {
    map_res(take_while1(is_digit), str_to_usize)(input)
        .map_err(|_: Err<()>| Err::Error(Error::msg(format!("could not parse '{}' as coordinate", input))))
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::TestResult;

    quickcheck! {
        fn parse_positive_coordinate(coord: usize) -> TestResult {
            let coord_str = coord.to_string();
            TestResult::from_bool(parse(&coord_str).unwrap() == ("", coord))
        }
    }

    quickcheck! {
        fn faile_parse_negative_coordinate(coord: i64) -> TestResult {
            if coord >= 0 {
                TestResult::discard()
            } else {
                let coord_str = coord.to_string();
                TestResult::from_bool(parse(&coord_str).is_err())
            }
        }
    }
}
