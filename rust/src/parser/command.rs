use super::simple::parse;
pub use crate::pixmap::Color;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Command {
    Help(HelpTopic),
    Size,
    PxGet(usize, usize),
    PxSet(usize, usize, Color),
    State(StateEncodingAlgorithm),
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse(s) {
            Ok((_remainder, cmd)) => Ok(cmd),
            Err(e) => match e {
                nom::Err::Error(e) => Err(e.into()),
                nom::Err::Failure(e) => Err(e.into()),
                nom::Err::Incomplete(_) => Err(anyhow::Error::msg("too little input")),
            },
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StateEncodingAlgorithm {
    Rgb64,
    Rgba64,
}
