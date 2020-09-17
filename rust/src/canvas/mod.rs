mod act_local_canvas;
mod msg_get_pixel;
//mod msg_get_size;
//mod msg_set_pixel;

pub use act_local_canvas::LocalCanvas;

pub type Color = [u8; 3];

pub mod messages {
    pub use super::msg_get_pixel::*;
    //pub use super::msg_get_size::*;
    //pub use super::msg_set_pixel::*;
}
