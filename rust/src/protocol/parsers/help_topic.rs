use super::Error;
use crate::protocol::HelpTopic;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::value;
use nom::{Err, IResult};

/// the topic that is given to HELP
pub(super) fn parse(input: &str) -> IResult<&str, HelpTopic, Error> {
    alt((
        value(HelpTopic::General, tag_no_case("help")),
        value(HelpTopic::Size, tag_no_case("size")),
        value(HelpTopic::Px, tag_no_case("px")),
        value(HelpTopic::State, tag_no_case("state")),
    ))(input)
    .map_err(|_: Err<()>| Err::Error(Error::msg(format!("there is no help for topic '{}'", input))))
}
