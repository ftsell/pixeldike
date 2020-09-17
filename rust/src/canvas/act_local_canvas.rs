use crate::actor_framework::*;
use crate::canvas::messages::*;
use crate::canvas::Color;

const DEFAULT_SIZE: (usize, usize) = (800, 600);
const MAILBOX_SIZE: usize = 128;

#[derive(Debug)]
pub struct LocalCanvas {
    data: Vec<Color>,
    width: usize,
    height: usize,
}

impl LocalCanvas {
    #[inline]
    fn coords_2_index(&self, x: usize, y: usize) -> usize {
        (self.width * y) + x
    }

    #[inline]
    fn are_coords_inside(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }
}

impl Default for LocalCanvas {
    fn default() -> Self {
        LocalCanvas {
            width: DEFAULT_SIZE.0,
            height: DEFAULT_SIZE.1,
            data: (0..(DEFAULT_SIZE.0 * DEFAULT_SIZE.1))
                .into_iter()
                .map(|_| [0, 0, 0])
                .collect(),
        }
    }
}

impl Handler<GetPixelMsg> for LocalCanvas {
    fn handle(&self, msg: GetPixelMsg) -> <GetPixelMsg as Message>::Response {
        todo!()
    }
}
