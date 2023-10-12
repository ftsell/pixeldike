//! Data types that describe all protocol interactions as safe-to-use structs

use crate::pixmap::Color;
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Request {
    Help(HelpTopic),
    GetSize,
    GetPixel { x: usize, y: usize },
    SetPixel { x: usize, y: usize, color: Color },
    GetState(StateEncodingAlgorithm),
}

impl Display for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Request::Help(topic) => match topic {
                HelpTopic::General => f.write_str("HELP"),
                HelpTopic::State => f.write_str("HELP STATE"),
                HelpTopic::Size => f.write_str("HELP SIZE"),
                HelpTopic::Px => f.write_str("HELP PX"),
            },
            Request::GetSize => f.write_str("SIZE"),
            Request::GetPixel { x, y } => f.write_fmt(format_args!("PX {} {}", x, y)),
            Request::SetPixel { x, y, color } => f.write_fmt(format_args!("PX {} {} #{:X}", x, y, color)),
            Request::GetState(alg) => f.write_fmt(format_args!("STATE {} ...", alg)),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Response<'data> {
    Help(HelpTopic),
    Size {
        width: usize,
        height: usize,
    },
    PxData {
        x: usize,
        y: usize,
        color: Color,
    },
    State {
        alg: StateEncodingAlgorithm,
        data: &'data [u8],
    },
}

impl Response<'_> {
    pub fn to_owned(&self) -> OwnedResponse {
        match self {
            Response::Help(topic) => OwnedResponse::Help(*topic),
            Response::Size { width, height } => OwnedResponse::Size {
                width: *width,
                height: *height,
            },
            Response::PxData { x, y, color } => OwnedResponse::PxData {
                x: *x,
                y: *y,
                color: *color,
            },
            Response::State { alg, data } => OwnedResponse::State {
                alg: *alg,
                data: Vec::from(*data),
            },
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OwnedResponse {
    Help(HelpTopic),
    Size {
        width: usize,
        height: usize,
    },
    PxData {
        x: usize,
        y: usize,
        color: Color,
    },
    State {
        alg: StateEncodingAlgorithm,
        data: Vec<u8>,
    },
}
