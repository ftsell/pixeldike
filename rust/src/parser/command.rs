pub use crate::pixmap::Color;
use nom::lib::std::str::FromStr;

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Command {
    Help(HelpTopic),
    Size,
    PxGet(usize, usize),
    PxSet(usize, usize, Color),
}

impl FromStr for Command {
    type Err = ();

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        todo!() // TODO Implement and use
    }
}
