use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::Display;

use super::Accumulator;
use super::CollisionCell;
use crate::Percentiles;
use crate::Recording;

/// Collision matrix across all recorded labels × candidate patterns
/// (TRAINING.md 2.4).
#[derive(Debug, Serialize, Deserialize)]
pub struct CollisionReport {
    /// Matrix cells with at least one tested frame, sorted by label ×
    /// pattern.
    pub cells: Vec<CollisionCell>,
}

impl CollisionReport {
    /// Computes the collision matrix for a loaded recording.
    ///
    /// For every labeled frame, each candidate pattern *other* than the
    /// labeled one contributes its best (lowest required-tolerance) candidate
    /// - [`FrameRecord::best_candidates`]. Patterns matching their own label
    ///   are skipped: those are true positives, not collisions.
    ///
    /// [`FrameRecord::best_candidates`]: tangible_objects_model::FrameRecord::best_candidates
    pub fn analyze(recording: &Recording) -> Self {
        let mut acc: BTreeMap<(String, String), Accumulator> = BTreeMap::new();

        for frame in recording.frames() {
            let Some(label) = &frame.label else { continue };
            for (pattern_name, best) in frame.best_candidates() {
                if best.matches_label(label) {
                    continue;
                }
                let entry = acc
                    .entry((label.as_str().to_string(), pattern_name.to_string()))
                    .or_insert_with(|| Accumulator::new(best.tolerance));
                entry.frames_tested += 1;
                if best.matched {
                    entry.collided_frames += 1;
                }
                entry
                    .required
                    .push(best.required_tolerance().as_millimeters());
            }
        }

        Self {
            cells: acc
                .into_iter()
                .map(|((label, pattern_name), entry)| CollisionCell {
                    label,
                    pattern_name,
                    frames_tested: entry.frames_tested,
                    collided_frames: entry.collided_frames,
                    required: Percentiles::of(entry.required.iter().copied()),
                    tolerance: entry.tolerance,
                })
                .collect(),
        }
    }
}

impl Display for CollisionReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "== Collision matrix ==")?;
        if self.cells.is_empty() {
            writeln!(f, "  (no labeled frames)")?;
            return Ok(());
        }
        for cell in &self.cells {
            let required = cell
                .required
                .map(|stats| {
                    format!(
                        "min={:.2} p50={:.2} p95={:.2}",
                        stats.min, stats.p50, stats.p95
                    )
                })
                .unwrap_or_else(|| "-".to_string());
            let flag = if cell.collided_frames.as_u64() > 0 {
                " ← COLLIDES"
            } else {
                ""
            };
            writeln!(
                f,
                "  {} → {}: {}/{} collided (tol {:.2}) | required: {}{}",
                cell.label,
                cell.pattern_name,
                cell.collided_frames,
                cell.frames_tested,
                cell.tolerance.as_f64(),
                required,
                flag
            )?;
        }
        Ok(())
    }
}
