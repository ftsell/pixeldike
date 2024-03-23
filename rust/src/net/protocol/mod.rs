//! Definitions for the network protocol

mod compliant_parser;
mod dtypes;

pub use dtypes::*;

pub use compliant_parser::parse_request_bin as parse_request;
pub use compliant_parser::parse_response_bin as parse_response;
