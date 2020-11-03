use crate::parser::command::*;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace1;
use nom::combinator::{all_consuming, value};
use nom::IResult;

pub(super) fn help_topic(input: &str) -> IResult<&str, HelpTopic> {
    alt((
        value(HelpTopic::General, tag_no_case("help")),
        value(HelpTopic::Size, tag_no_case("size")),
        value(HelpTopic::Px, tag_no_case("px")),
        value(HelpTopic::State, tag_no_case("state")),
    ))(input)
}

pub(super) fn whitespace(input: &str) -> IResult<&str, ()> {
    // parse one or more spacing characters but simply discard them
    let (input, _) = multispace1(input)?;
    Ok((input, ()))
}

#[derive(Debug, Copy, Clone)]
pub(super) enum PrimaryCommand {
    Help,
    Size,
    Px,
}

pub(super) fn primary_command(input: &str) -> IResult<&str, PrimaryCommand> {
    alt((
        value(PrimaryCommand::Help, tag_no_case("help")),
        value(PrimaryCommand::Size, tag_no_case("size")),
        value(PrimaryCommand::Px, tag_no_case("px")),
    ))(input)
}
