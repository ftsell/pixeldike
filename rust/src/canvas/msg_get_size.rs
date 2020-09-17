use actix::prelude::*;
use derive_more::{Display, Constructor};

#[derive(Constructor, Copy, Clone, Debug)]
pub struct GetSizeMsg {}

pub type GetSizeResult = MessageResult<GetSizeMsg>;

impl Message for GetSizeMsg {
    type Result = (usize, usize);
}
