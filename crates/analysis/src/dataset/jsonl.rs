use std::path::Path;

use serde_jsonlines::write_json_lines;

use super::DatasetRow;
use crate::AnalysisError;
use crate::Recording;

/// Labeled dataset over a whole recording - one row per candidate of every
/// labeled frame (TRAINING.md 3.3).
///
/// Serialized as JSONL - the same line format as the recordings themselves:
/// typed fields instead of CSV's untyped, escaping- and locale-sensitive
/// text cells. Unlabeled frames contribute no rows: without a ground-truth
/// target the features are unlearnable.
#[derive(Debug)]
pub struct Dataset {
    /// Collected candidate rows.
    pub rows: Vec<DatasetRow>,
    /// Width of the widest feature vector in the dataset.
    pub feature_width: usize,
}

impl Dataset {
    /// Collects the dataset from labeled frames of a recording.
    pub fn of(recording: &Recording) -> Self {
        let mut rows = Vec::new();
        for frame in recording.frames() {
            let Some(label) = &frame.label else { continue };
            for candidate in &frame.candidates {
                if candidate.deviations.is_empty() {
                    continue;
                }
                rows.push(DatasetRow::of(frame, label.as_str(), candidate));
            }
        }
        let feature_width = rows.iter().map(|row| row.features.len()).max().unwrap_or(0);
        Self {
            rows,
            feature_width,
        }
    }

    /// Writes the dataset as JSONL - one [`DatasetRow`] per line.
    pub fn write_jsonl(&self, path: &Path) -> Result<(), AnalysisError> {
        write_json_lines(path, &self.rows).map_err(|source| AnalysisError::Io {
            path: path.to_path_buf(),
            source,
        })
    }
}
