//! TOML-serializable figure profiles.
//!
//! [`ProfileFile`] is the on-disk representation of a figure fingerprint,
//! [`ProfileDir`] collects the outcome of loading all profiles from a
//! directory and [`ProfileError`] describes load/save failures.

mod dir;
mod error;
mod file;

pub use dir::ProfileDir;
pub use error::ProfileError;
pub use file::ProfileFile;
