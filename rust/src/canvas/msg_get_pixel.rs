use crate::actor_framework::*;
use crate::canvas::Color;
use derive_more::{Constructor, Display};

#[derive(Constructor, Copy, Clone, Debug)]
pub struct GetPixelMsg {
    pub x: usize,
    pub y: usize,
}

pub type GetPixelResult = Result<Color, GetPixelError>;

#[derive(Display, Debug, Copy, Clone)]
pub enum GetPixelError {
    #[display(fmt = "coordinates are not inside the canvas")]
    CoordinatesNotInside,
}

impl Message for GetPixelMsg {
    type Response = GetPixelResult;
}
