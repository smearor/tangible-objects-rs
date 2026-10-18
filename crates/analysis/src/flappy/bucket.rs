use serde::Serialize;
use tangible_objects_model::ContactCount;
use tangible_objects_model::FrameCount;

/// One bucket of the contact-lifetime histogram.
#[derive(Debug, Serialize)]
pub struct LifetimeBucket {
    /// Inclusive upper bound in frames; `None` marks the open-ended top
    /// bucket (`> last edge`).
    pub upper_bound: Option<FrameCount>,
    /// Contacts whose lifetime fell into this bucket.
    pub count: ContactCount,
}
