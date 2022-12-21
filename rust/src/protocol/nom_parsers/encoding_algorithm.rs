use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::value;
use nom::{Err, IResult};

use crate::protocol::StateEncodingAlgorithm;

use super::Error;

/// a key which can be given to STATE to specify the encoding algorithm
pub(super) fn parse(input: &str) -> IResult<&str, StateEncodingAlgorithm, Error> {
    alt((
        value(StateEncodingAlgorithm::Rgb64, tag_no_case("rgb64")),
        value(StateEncodingAlgorithm::Rgba64, tag_no_case("rgba64")),
    ))(input)
    .map_err(|_: Err<()>| {
        Err::Error(Error::msg(format!(
            "'{}' is not a supported state encoding algorithm",
            input
        )))
    })
}
