mod color;
mod coordinates;
mod help_topic;
mod response;
mod server_config;

#[cfg(debug_assertions)]
use nom::error::VerboseError;

pub use response::parse_response;

#[cfg(debug_assertions)]
type ProtocolError<'a> = VerboseError<&'a [u8]>;
#[cfg(not(debug_assertions))]
type ProtocolError<'a> = ();
