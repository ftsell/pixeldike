use crate::net::protocol::dtypes::ServerConfig;
use crate::net::protocol::nom_parsers::ProtocolError;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::u64;
use nom::sequence::separated_pair;
use nom::IResult;

fn parse_key_value<'a, 'b>(
    input: &'a [u8],
    expected_key: &'b str,
) -> IResult<&'a [u8], usize, ProtocolError<'a>> {
    let (remainder, (_, value)) = separated_pair(tag_no_case(expected_key), tag("="), u64)(input)?;
    Ok((remainder, value as usize))
}

/// Parse a valid server configuration object from the given data
pub(super) fn parse_server_config(input: &[u8]) -> IResult<&[u8], ServerConfig, ProtocolError> {
    let (input, max_udp_packet_size) = parse_key_value(input, "max_udp_packet_size")?;
    Ok((input, ServerConfig { max_udp_packet_size }))
}
