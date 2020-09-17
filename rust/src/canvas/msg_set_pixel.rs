use actix::prelude::*;
use derive_more::{Display, Constructor};
use crate::canvas::Color;

#[derive(Constructor, Copy, Clone, Debug)]
pub struct SetPixelMsg {
    pub x: usize,
    pub y: usize,
    pub color: Color,
}

pub type SetPixelResult = Result<(), SetPixelError>;

#[derive(Display, Debug, Copy, Clone)]
pub enum SetPixelError {
    #[display(fmt = "coordinates are not inside the canvas")]
    CoordinatesNotInside,
}

impl Message for SetPixelMsg {
    type Result = SetPixelResult;
}
