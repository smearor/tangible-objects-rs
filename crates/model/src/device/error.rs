use std::path::PathBuf;

/// Error type for device-profile loading.
#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
    /// Reading a device profile file failed.
    #[error("cannot read device profile {path}: {source}")]
    Io {
        /// Profile path that failed to open or read.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A profile file did not parse into a [`super::DeviceProfile`].
    #[error("device profile {path} is malformed: {source}")]
    Parse {
        /// Path of the malformed profile.
        path: PathBuf,
        /// Underlying TOML error.
        source: toml::de::Error,
    },
}
