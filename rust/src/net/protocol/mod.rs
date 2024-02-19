//! Definitions for the network protocol

mod compliant_parser;
mod dtypes;
mod nom_parsers;

pub use dtypes::*;
//pub use nom_parsers::{parse_request, parse_response};
pub use nom_parsers::parse_response;

pub use compliant_parser::parse_request_slice as parse_request;
