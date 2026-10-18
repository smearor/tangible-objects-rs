use std::path::PathBuf;

use super::ProfileError;
use super::ProfileFile;
use crate::PatternId;
use crate::TangiblePattern;

/// Outcome of [`ProfileFile::load_dir`]: successfully loaded profiles plus
/// the files that failed (path + error) so callers can warn about them.
#[derive(Debug, Default)]
pub struct ProfileDir {
    /// Loaded profiles with their source file paths.
    pub profiles: Vec<(PathBuf, ProfileFile)>,
    /// Files that failed to load with their errors.
    pub errors: Vec<(PathBuf, ProfileError)>,
}

impl ProfileDir {
    /// Converts all loaded profiles into matchable [`TangiblePattern`]s with
    /// sequential IDs starting at 1 (deterministic, sorted by filename).
    /// Disabled profiles (`enabled = false`) are skipped.
    pub fn into_patterns(self) -> Vec<TangiblePattern> {
        self.profiles
            .iter()
            .filter(|(_, profile)| profile.enabled)
            .enumerate()
            .map(|(index, (_, profile))| profile.to_pattern(PatternId::new(index as u32 + 1)))
            .collect()
    }
}
