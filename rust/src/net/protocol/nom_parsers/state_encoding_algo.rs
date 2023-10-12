use crate::net::protocol::nom_parsers::ProtocolError;
use crate::net::protocol::StateEncodingAlgorithm;
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::combinator::value;
use nom::IResult;

/// Parse a valid state encoding algorithm from the input
pub(super) fn parse_state_encoding_algo(
    input: &[u8],
) -> IResult<&[u8], StateEncodingAlgorithm, ProtocolError> {
    alt((
        value(StateEncodingAlgorithm::Rgb64, tag_no_case("rgb64")),
        value(StateEncodingAlgorithm::Rgba64, tag_no_case("rgba64")),
    ))(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_algorithm() {
        let (remainder, algo) = parse_state_encoding_algo("RGB64".as_bytes()).unwrap();
        assert_eq!(remainder.len(), 0);
        assert_eq!(algo, StateEncodingAlgorithm::Rgb64);
    }

    #[test]
    fn test_invalid_algorithm() {
        let result = parse_state_encoding_algo("mp4".as_bytes());
        assert!(result.is_err());
    }
}
