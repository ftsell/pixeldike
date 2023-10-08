use crate::net_protocol::dtypes::{HelpTopic, Request};
use crate::net_protocol::nom_parsers::{
    parse_color, parse_coordinate, parse_help_topic, parse_state_encoding_algo, ProtocolError,
};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::space1;
use nom::combinator::{eof, flat_map, map, value};
use nom::sequence::{pair, preceded};
use nom::IResult;

/// Parse a complete request and return the encoded form
pub fn parse_request(input: &[u8]) -> IResult<&[u8], Request, ProtocolError> {
    alt((
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
        // STATE
        preceded(
            tag_no_case("state"),
            preceded(space1, map(parse_state_encoding_algo, Request::GetState)),
        ),
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
                        // PX $X $Y
                        value(Request::GetPixel { x, y }, eof),
                        // PX $X $Y $COLOR
                        preceded(
                            space1,
                            map(parse_color, move |color| Request::SetPixel { x, y, color }),
                        ),
                    ))
                },
            ),
        ),
    ))(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::byte_protocol::dtypes::StateEncodingAlgorithm;
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
}
