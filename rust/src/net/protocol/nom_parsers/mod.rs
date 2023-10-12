mod color;
mod coordinates;
mod help_topic;
mod request;
mod response;
mod state_encoding_algo;

use color::parse_color;
use coordinates::parse_coordinate;
use help_topic::parse_help_topic;
use nom::error::VerboseError;
use state_encoding_algo::parse_state_encoding_algo;

pub use request::parse_request;
pub use response::parse_response;

#[cfg(debug_assertions)]
type ProtocolError<'a> = VerboseError<&'a [u8]>;
#[cfg(not(debug_assertions))]
type ProtocolError = ();
