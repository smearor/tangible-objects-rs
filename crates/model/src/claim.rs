use serde::Deserialize;
use serde::Serialize;

use crate::RefIndex;

/// One completion claim: a measured touch bound to the reference pin
/// it currently fills (TRACKING_STABILIZATION.md, ROTATION.md).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    /// Position in the frame's touch point list.
    pub touch_index: usize,
    /// Reference pin the claimed touch fills.
    pub ref_index: RefIndex,
}

impl Claim {
    /// A claim pair.
    pub fn new(touch_index: usize, ref_index: RefIndex) -> Self {
        Self {
            touch_index,
            ref_index,
        }
    }
}

impl From<(usize, usize)> for Claim {
    /// `(touch, ref)` tuple - convenience for test literals.
    fn from((touch_index, ref_index): (usize, usize)) -> Self {
        Self::new(touch_index, ref_index.into())
    }
}
