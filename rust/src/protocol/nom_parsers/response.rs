use nom::branch::{alt, permutation};
use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace1;
use nom::combinator::{cut, map};
use nom::IResult;

use crate::protocol::Response;

use super::combinators::*;
use super::Error;
use super::{coordinate, encoding_algorithm, hex_color};

pub fn parse(input: &str) -> IResult<&str, Response, Error> {
    Ok(alt((
        cond_parse(
            tag_no_case("size"),
            cut(map(
                permutation((multispace1, coordinate::parse, multispace1, coordinate::parse)),
                |(_, width, _, height)| Response::Size(width, height),
            )),
        ),
        cond_parse(
            tag_no_case("px"),
            cut(map(
                permutation((
                    multispace1,
                    coordinate::parse,
                    multispace1,
                    coordinate::parse,
                    multispace1,
                    hex_color::parse,
                )),
                |(_, x, _, y, _, color)| Response::Px(x, y, color),
            )),
        ),
        cond_parse(
            tag_no_case("state"),
            cut(map(
                permutation((
                    multispace1,
                    encoding_algorithm::parse,
                    multispace1,
                    |input: &str| Ok(("", input.to_string())),
                )),
                |(_, alg, _, data)| Response::State(alg, data),
            )),
        ),
    ))(input)?)
}
