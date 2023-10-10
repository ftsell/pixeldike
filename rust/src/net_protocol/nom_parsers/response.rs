use crate::net_protocol::dtypes::Response;
use crate::net_protocol::nom_parsers::color::parse_color;
use crate::net_protocol::nom_parsers::{parse_coordinate, parse_state_encoding_algo, ProtocolError};
use nom::branch::{alt, permutation};
use nom::bytes::complete::tag_no_case;
use nom::character::complete::space1;
use nom::combinator::{map, rest};
use nom::sequence::{pair, preceded};
use nom::IResult;

/// Parse a complete request and return the encoded form
#[tracing::instrument(skip_all)]
pub fn parse_response(input: &[u8]) -> IResult<&[u8], Response, ProtocolError> {
    alt((
        // SIZE $WIDTH $HEIGHT
        preceded(
            tag_no_case("size"),
            map(
                pair(
                    preceded(space1, parse_coordinate),
                    preceded(space1, parse_coordinate),
                ),
                |(width, height)| Response::Size { width, height },
            ),
        ),
        // PX $X $Y $COLOR
        preceded(
            tag_no_case("px"),
            map(
                permutation((
                    preceded(space1, parse_coordinate),
                    preceded(space1, parse_coordinate),
                    preceded(space1, parse_color),
                )),
                |(x, y, color)| Response::PxData { x, y, color },
            ),
        ),
        // STATE
        preceded(
            tag_no_case("state"),
            map(
                pair(
                    preceded(space1, parse_state_encoding_algo),
                    preceded(space1, rest),
                ),
                |(alg, data)| Response::State { alg, data },
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
    fn test_size_data() {
        let (remainder, response) = parse_response("SIZE 800 600".as_bytes()).unwrap();
        assert_eq!(remainder.len(), 0);
        assert_eq!(
            response,
            Response::Size {
                width: 800,
                height: 600
            }
        );
    }

    #[test]
    fn test_state_data() {
        let (remainder, response) = parse_response("STATE RGBA64 foobar123".as_bytes()).unwrap();
        assert_eq!(remainder.len(), 0);
        assert_eq!(
            response,
            Response::State {
                alg: StateEncodingAlgorithm::Rgba64,
                data: "foobar123".as_bytes()
            }
        );
    }

    #[test]
    fn test_pixel_data() {
        let (remainder, response) = parse_response("PX 42 112 #FF00AA".as_bytes()).unwrap();
        assert_eq!(remainder.len(), 0);
        assert_eq!(
            response,
            Response::PxData {
                x: 42,
                y: 112,
                color: Color(0xFF, 0x00, 0xAA)
            }
        );
    }
}
