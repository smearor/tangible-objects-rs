/// Errors that can occur while loading or saving figure profiles.
#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    /// An I/O error while reading or writing a profile file.
    #[error("profile I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The TOML content could not be deserialized into a profile.
    #[error("invalid profile TOML: {0}")]
    Deserialize(#[from] toml::de::Error),

    /// The profile could not be serialized to TOML.
    #[error("profile serialization failed: {0}")]
    Serialize(#[from] toml::ser::Error),
}
