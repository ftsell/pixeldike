use std::sync::{Arc, Mutex};

use super::*;

/// A [`MultiEncodings`] that can be shared
pub type SharedMultiEncodings = Arc<MultiEncodings>;

/// Data structure to hold pixmap data in all known encoding algorithms
#[derive(Debug)]
pub struct MultiEncodings {
    /// A pixmap canvas encoded via the *RGB64* algorithm
    pub rgb64: Mutex<rgb64::Encoding>,
    /// A pixmap canvas encoded via the *RGBA64* algorithm
    pub rgba64: Mutex<rgba64::Encoding>,
}

impl MultiEncodings {
    /// Create a new MultiEncoding with empty data
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
