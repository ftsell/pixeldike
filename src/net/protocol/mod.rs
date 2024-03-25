//! Definitions for the network protocol

mod compliant_parser;
mod dtypes;

pub use dtypes::*;

pub use compliant_parser::{parse_request_bin, parse_request_str};
pub use compliant_parser::{parse_response_bin, parse_response_str};
