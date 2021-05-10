//!
//! Data structures for pixelflut responses received from a server
//!

use crate::i18n::get_catalog;
use crate::pixmap::Color;
use crate::protocol::{HelpTopic, StateEncodingAlgorithm};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Response {
    Help(HelpTopic),
    Size(usize, usize),
    Px(usize, usize, Color),
    State(StateEncodingAlgorithm, String),
}

impl FromStr for Response {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Response::Help(topic) => match topic {
                HelpTopic::General => f.write_str(&i18n!(get_catalog(), "help_general")),
                HelpTopic::Size => f.write_str(&i18n!(get_catalog(), "help_size")),
                HelpTopic::Px => f.write_str(&i18n!(get_catalog(), "help_px")),
                HelpTopic::State => f.write_str(&i18n!(get_catalog(), "help_state")),
            },
            Response::Size(width, height) => f.write_fmt(format_args!("SIZE {} {}", width, height)),
            Response::Px(x, y, color) => f.write_fmt(format_args!("PX {} {} #{:X}", x, y, color)),
            Response::State(alg, data) => f.write_fmt(format_args!("STATE {} {}", alg, data)),
        }
    }
}
