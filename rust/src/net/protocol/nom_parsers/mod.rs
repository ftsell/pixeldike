mod color;
mod coordinates;
mod help_topic;
mod request;
mod response;
mod server_config;

use nom::error::VerboseError;

pub use request::parse_request;
pub use response::parse_response;

#[cfg(debug_assertions)]
type ProtocolError<'a> = VerboseError<&'a [u8]>;
#[cfg(not(debug_assertions))]
type ProtocolError<'a> = ();
