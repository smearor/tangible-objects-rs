use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

use tangible_objects_model::FrameRecord;
use tangible_objects_model::RECORD_SCHEMA_VERSION;

use crate::AnalysisError;

/// A loaded frame recording: validated JSONL lines of schema `"v": 1`.
///
/// Loading rejects malformed lines and unknown schema versions up front -
/// every returned frame is a parseable, version-checked [`FrameRecord`].
#[derive(Debug)]
pub struct Recording {
    frames: Vec<FrameRecord>,
}

impl Recording {
    /// Loads and validates a `.jsonl` recording produced by `--record`.
    ///
    /// Empty lines are skipped; every other line must parse as a
    /// [`FrameRecord`] with schema version [`RECORD_SCHEMA_VERSION`].
    pub fn load(path: &Path) -> Result<Self, AnalysisError> {
        let file = File::open(path).map_err(|source| AnalysisError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let mut frames = Vec::new();
        for (index, line) in BufReader::new(file).lines().enumerate() {
            let line_no = index + 1;
            let line = line.map_err(|source| AnalysisError::Io {
                path: PathBuf::from(path),
                source,
            })?;
            if line.trim().is_empty() {
                continue;
            }
            let frame: FrameRecord =
                serde_json::from_str(&line).map_err(|source| AnalysisError::Parse {
                    line: line_no,
                    source,
                })?;
            if frame.schema_version != RECORD_SCHEMA_VERSION {
                return Err(AnalysisError::UnsupportedVersion {
                    line: line_no,
                    found: frame.schema_version,
                    expected: RECORD_SCHEMA_VERSION,
                });
            }
            frames.push(frame);
        }
        Ok(Self { frames })
    }

    /// All recorded frames in file order.
    pub fn frames(&self) -> &[FrameRecord] {
        &self.frames
    }
}

#[cfg(test)]
mod tests {
    use super::Recording;
    use crate::AnalysisError;

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("robofigure-analysis-{name}-{}", std::process::id()))
    }

    fn write_lines(path: &std::path::Path, lines: &[&str]) -> std::io::Result<()> {
        std::fs::write(path, lines.join("\n") + "\n")
    }

    #[test]
    fn loads_valid_recording() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_path("valid.jsonl");
        write_lines(
            &path,
            &[
                r#"{"v":1,"frame":0,"timestamp_ms":0,"label":null,"calibration":null,"touches":[],"candidates":[],"tracked":[],"raw_dropped":0}"#,
                "",
                r#"{"v":1,"frame":1,"timestamp_ms":8,"label":"fig","calibration":null,"touches":[],"candidates":[],"tracked":[],"raw_dropped":0}"#,
            ],
        )?;
        let recording = Recording::load(&path)?;
        assert_eq!(recording.frames().len(), 2);
        assert_eq!(recording.frames()[1].frame.0, 1);
        std::fs::remove_file(&path).ok();
        Ok(())
    }

    #[test]
    fn rejects_unknown_schema_version() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_path("bad-version.jsonl");
        write_lines(
            &path,
            &[
                r#"{"v":2,"frame":0,"timestamp_ms":0,"label":null,"calibration":null,"touches":[],"candidates":[],"tracked":[],"raw_dropped":0}"#,
            ],
        )?;
        assert!(matches!(
            Recording::load(&path),
            Err(AnalysisError::UnsupportedVersion {
                line: 1,
                found: 2,
                ..
            })
        ));
        std::fs::remove_file(&path).ok();
        Ok(())
    }

    #[test]
    fn rejects_malformed_line() -> Result<(), Box<dyn std::error::Error>> {
        let path = temp_path("malformed.jsonl");
        write_lines(&path, &["{not json"])?;
        assert!(matches!(
            Recording::load(&path),
            Err(AnalysisError::Parse { line: 1, .. })
        ));
        std::fs::remove_file(&path).ok();
        Ok(())
    }
}
