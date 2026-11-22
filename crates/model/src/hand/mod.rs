//! `hand` - the handprint registry (SPELLS.md's `HandProfile`): a
//! five-contact constellation recorded as a pattern-like profile so
//! `five-finger-tap` arming can bind spells to a specific user's
//! hand (per-user spells).

mod name;
mod profile;

pub use name::HandName;
pub use profile::HandProfile;
