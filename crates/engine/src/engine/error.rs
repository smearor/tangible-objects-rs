/// Errors produced by the detection engine (device access and replay I/O).
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// An I/O error while opening or reading a device or replay file.
    #[error("engine I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// The input could not be converted or interpreted.
    #[error("{0}")]
    InvalidInput(String),
}
