use super::SimpleError;
use nom::bytes::complete::take_while1;
use nom::combinator::map_res;
use nom::Err;
use nom::IResult;
use std::num::ParseIntError;

fn is_digit(c: char) -> bool {
    c.is_digit(10)
}

fn str_to_usize(input: &str) -> Result<usize, ParseIntError> {
    usize::from_str_radix(input, 10)
}

/// a canvas coordinate represented as a decimal digit
pub(super) fn coordinate(input: &str) -> IResult<&str, usize, SimpleError> {
    map_res(take_while1(is_digit), str_to_usize)(input)
        .map_err(|_: Err<()>| Err::Error(SimpleError::Coordinate(input.to_string())))
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::TestResult;

    quickcheck! {
        fn parse_positive_coordinate(coord: usize) -> TestResult {
            let coord_str = coord.to_string();
            TestResult::from_bool(coordinate(&coord_str) == Ok(("", coord)))
        }
    }

    quickcheck! {
        fn faile_parse_negative_coordinate(coord: i64) -> TestResult {
            if coord >= 0 {
                TestResult::discard()
            } else {
                let coord_str = coord.to_string();
                TestResult::from_bool(coordinate(&coord_str).is_err())
            }
        }
    }
}
