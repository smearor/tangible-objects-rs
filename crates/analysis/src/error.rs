use std::path::PathBuf;

/// Error type of the analysis crate.
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    /// Reading the recording file failed.
    #[error("cannot read recording {path}: {source}")]
    Io {
        /// Recording path that failed to open or read.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A JSONL line failed to parse as a `FrameRecord`.
    #[error("line {line} of the recording is malformed: {source}")]
    Parse {
        /// 1-based line number of the malformed record.
        line: usize,
        /// Underlying JSON error.
        source: serde_json::Error,
    },
    /// The recording uses a schema version this analyzer does not understand.
    #[error("line {line}: unsupported schema version {found} (expected {expected})")]
    UnsupportedVersion {
        /// 1-based line number of the offending record.
        line: usize,
        /// Schema version found in the record.
        found: u16,
        /// Schema version this analyzer supports.
        expected: u16,
    },
}
