use crate::net::protocol::nom_parsers::color::parse_color;
use crate::net::protocol::nom_parsers::coordinates::parse_coordinate;
use crate::net::protocol::nom_parsers::help_topic::parse_help_topic;
use crate::net::protocol::nom_parsers::ProtocolError;
use crate::net::protocol::{HelpTopic, Request};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::space1;
use nom::combinator::{eof, flat_map, map, value};
use nom::sequence::{pair, preceded};
use nom::IResult;

/// Parse a complete request and return the encoded form
#[tracing::instrument(skip_all)]
pub fn parse_request(input: &[u8]) -> IResult<&[u8], Request, ProtocolError> {
    alt((
        // PX
        preceded(
            tag_no_case("px"),
            flat_map(
                pair(
                    preceded(space1, parse_coordinate),
                    preceded(space1, parse_coordinate),
                ),
                |(x, y)| {
                    alt((
                        // PX $X $Y $COLOR
                        preceded(
                            space1,
                            map(parse_color, move |color| Request::SetPixel { x, y, color }),
                        ),
                        // PX $X $Y
                        value(Request::GetPixel { x, y }, eof),
                    ))
                },
            ),
        ),
        // HELP
        preceded(
            tag_no_case("help"),
            alt((
                // HELP $TOPIC
                preceded(space1, map(parse_help_topic, Request::Help)),
                // HELP (no topic)
                value(Request::Help(HelpTopic::General), eof),
            )),
        ),
        // SIZE
        value(Request::GetSize, tag_no_case("size")),
        // CONFIG
        value(Request::GetConfig, tag_no_case("config")),
    ))(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::net::protocol::StateEncodingAlgorithm;
    use crate::pixmap::Color;

    #[test]
    fn test_help_help_no_topic() {
        let (remainder, request) = parse_request("HELP".as_bytes()).unwrap();
        assert_eq!(remainder.len(), 0);
        assert_eq!(request, Request::Help(HelpTopic::General));
    }

    #[test]
    fn test_help_help_with_topic() {
        let (remainder, request) = parse_request("HELP PX".as_bytes()).unwrap();
        assert_eq!(remainder.len(), 0);
        assert_eq!(request, Request::Help(HelpTopic::Px));
    }

    #[test]
    fn test_get_size() {
        let (remainder, request) = parse_request("SIZE".as_bytes()).unwrap();
        assert_eq!(remainder.len(), 0);
        assert_eq!(request, Request::GetSize);
    }

    #[test]
    fn test_get_state() {
        let (remainder, request) = parse_request("STATE RGBA64".as_bytes()).unwrap();
        assert_eq!(remainder.len(), 0);
        assert_eq!(request, Request::GetState(StateEncodingAlgorithm::Rgba64));
    }

    #[test]
    fn test_get_pixel() {
        let (remainder, request) = parse_request("PX 42 112".as_bytes()).unwrap();
        assert_eq!(remainder.len(), 0);
        assert_eq!(request, Request::GetPixel { x: 42, y: 112 });
    }

    #[test]
    fn test_set_pixel() {
        let (remainder, request) = parse_request("PX 42 112 FF00AA".as_bytes()).unwrap();
        assert_eq!(remainder.len(), 0);
        assert_eq!(
            request,
            Request::SetPixel {
                x: 42,
                y: 112,
                color: Color(0xFF, 0x00, 0xAA)
            }
        );
    }

    #[test]
    fn test_get_config() {
        let (remainder, request) = parse_request("CONFIG".as_bytes()).unwrap();
        assert_eq!(remainder.len(), 0);
        assert_eq!(request, Request::GetConfig,)
    }
}
