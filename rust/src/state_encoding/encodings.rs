use super::*;
use bytes::Bytes;
use std::sync::{Arc, Mutex};

pub type SharedMultiEncodings = Arc<MultiEncodings>;

pub struct MultiEncodings {
    pub rgb64: Mutex<rgb64::Encoding>,
    pub rgba64: Mutex<rgba64::Encoding>,
}

impl MultiEncodings {
    pub fn new() -> Self {
        MultiEncodings {
            rgb64: Mutex::new(rgb64::Encoding::new()),
            rgba64: Mutex::new(rgba64::Encoding::new()),
        }
    }
}

impl Default for MultiEncodings {
    fn default() -> Self {
        MultiEncodings::new()
    }
}
