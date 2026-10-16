/// One bucket of the contact-lifetime histogram.
#[derive(Debug)]
pub struct LifetimeBucket {
    /// Inclusive upper bound in frames; `None` marks the open-ended top
    /// bucket (`> last edge`).
    pub upper_bound: Option<u64>,
    /// Contacts whose lifetime fell into this bucket.
    pub count: usize,
}
