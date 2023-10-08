use crate::net_protocol::dtypes::HelpTopic;
use crate::net_protocol::nom_parsers::ProtocolError;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::value;
use nom::IResult;

/// Parse a valid help topic from the input slice
pub(super) fn parse_help_topic(input: &[u8]) -> IResult<&[u8], HelpTopic, ProtocolError> {
    alt((
        value(HelpTopic::General, tag_no_case("help")),
        value(HelpTopic::Size, tag_no_case("size")),
        value(HelpTopic::Px, tag_no_case("px")),
        value(HelpTopic::State, tag_no_case("state")),
    ))(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_topic() {
        let (remainder, topic) = parse_help_topic("size".as_bytes()).unwrap();
        assert_eq!(remainder.len(), 0);
        assert_eq!(topic, HelpTopic::Size);
    }

    #[test]
    fn test_error() {
        let err = parse_help_topic("bla".as_bytes());
        assert!(err.is_err());
    }
}
