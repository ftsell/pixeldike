use super::simple::parse;
pub use crate::pixmap::Color;
use anyhow::Error;
use nom::lib::std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Command {
    Help(HelpTopic),
    Size,
    PxGet(usize, usize),
    PxSet(usize, usize, Color),
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse(s) {
            Ok((_remainder, cmd)) => Ok(cmd),
            Err(_e) => Err(Error::msg("could not convert error :(")), // TODO handle and return error correctly
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
pub enum StateAlgorithm {
    Rgb64,
    Rgba64,
}
