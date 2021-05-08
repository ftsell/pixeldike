use super::SimpleError;
use crate::parser::command::*;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace1;
use nom::combinator::value;
use nom::Err;
use nom::IResult;

/// the topic that is given to HELP
pub(super) fn help_topic(input: &str) -> IResult<&str, HelpTopic, SimpleError> {
    alt((
        value(HelpTopic::General, tag_no_case("help")),
        value(HelpTopic::Size, tag_no_case("size")),
        value(HelpTopic::Px, tag_no_case("px")),
        value(HelpTopic::State, tag_no_case("state")),
    ))(input)
    .map_err(|_: Err<()>| Err::Error(SimpleError::HelpTopic(input.to_string())))
}

/// one ore more spacing characters (whitespace, tabs, …) which are discarded
pub(super) fn whitespace(input: &str) -> IResult<&str, (), SimpleError> {
    let (input, _) = multispace1(input)?;
    Ok((input, ()))
}

#[derive(Debug, Copy, Clone)]
pub(super) enum PrimaryCommand {
    Help,
    Size,
    Px,
}

/// the first word of a pixelflut command like HELP, SIZE, PX, etc.
pub(super) fn primary_command(input: &str) -> IResult<&str, PrimaryCommand, SimpleError> {
    alt((
        value(PrimaryCommand::Help, tag_no_case("help")),
        value(PrimaryCommand::Size, tag_no_case("size")),
        value(PrimaryCommand::Px, tag_no_case("px")),
    ))(input)
    .map_err(|_: Err<()>| Err::Error(SimpleError::PrimaryCommand(input.to_string())))
}
