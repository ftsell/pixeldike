use std::fmt::{Display, Formatter};

pub use request::Request;
pub use response::Response;

mod parsers;
mod request;
mod response;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StateEncodingAlgorithm {
    Rgb64,
    Rgba64,
}

impl Display for StateEncodingAlgorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StateEncodingAlgorithm::Rgb64 => f.write_str("RGB64"),
            StateEncodingAlgorithm::Rgba64 => f.write_str("RGBA64"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum HelpTopic {
    General,
    Size,
    Px,
    State,
}
