use nom::IResult;

use crate::net::protocol::nom_parsers::ProtocolError;
use nom::character::complete::u64;

/// Parse two digits separated by a space into a `(usize, usize)` tuple
pub(super) fn parse_coordinate(input: &[u8]) -> IResult<&[u8], usize, ProtocolError> {
    let (input, value) = u64(input)?;
    Ok((input, value as usize))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_coordinates() {
        let input = "128".as_bytes();
        let (remainder, coords) = parse_coordinate(input).unwrap();
        assert_eq!(remainder.len(), 0);
        assert_eq!(coords, 128);
    }

    #[test]
    fn test_error() {
        let err = parse_coordinate("bla".as_bytes());
        assert!(err.is_err());
    }
}
