use super::simple::parse;
pub use crate::pixmap::Color;
use std::fmt::{Display, Formatter};
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

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Help(topic) => match topic {
                HelpTopic::General => f.write_str("HELP"),
                HelpTopic::State => f.write_str("HELP STATE"),
                HelpTopic::Size => f.write_str("HELP SIZE"),
                HelpTopic::Px => f.write_str("HELP PX"),
            },
            Command::Size => f.write_str("SIZE"),
            Command::PxGet(x, y) => f.write_fmt(format_args!("PX {} {}", x, y)),
            Command::PxSet(x, y, color) => f.write_fmt(format_args!("PX {} {} #{:X}", x, y, color)),
            Command::State(alg) => match alg {
                StateEncodingAlgorithm::Rgb64 => f.write_str("STATE RGB64"),
                StateEncodingAlgorithm::Rgba64 => f.write_str("STATE RGBA64"),
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

#[cfg(test)]
#[test]
fn test_from_string_to_string() {
    macro_rules! assert_parsing {
        ($cmd:literal) => {
            assert_eq!($cmd, Command::from_str($cmd).unwrap().to_string());
        };
    }

    assert_parsing!("HELP");
    assert_parsing!("HELP STATE");
    assert_parsing!("HELP SIZE");
    assert_parsing!("HELP PX");
    assert_parsing!("SIZE");
    assert_parsing!("PX 42 42");
    assert_parsing!("PX 42 42 #890ABC");
    assert_parsing!("STATE RGB64");
    assert_parsing!("STATE RGBA64");
}
