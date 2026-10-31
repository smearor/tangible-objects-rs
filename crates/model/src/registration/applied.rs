use crate::PatternId;

/// The applied outcome of a [`crate::Registration`] - the
/// `Register` reply-channel payload so callers learn *what* was
/// registered (for profiles: the assigned pattern ID), not just that it
/// succeeded.
#[derive(Debug, Clone, Copy)]
pub enum RegistrationApplied {
    /// A profile registered - carries the assigned pattern ID.
    Profile(PatternId),
    /// A field set registered - the name was already known by the
    /// caller.
    FieldSet,
}
