use tangible_objects_model::PatternMatch;
use tangible_objects_model::RotationRad;
use tangible_objects_model::TrackedTangible;
use tangible_objects_model::Vec2;

/// Frame-to-frame tracker using nearest-neighbor association and EMA
/// smoothing.
///
/// Associates each frame's [`PatternMatch`]es with already tracked tangibles
/// (same pattern ID, bounded center distance), smooths pose with an
/// exponential moving average and expires tangibles after a configurable
/// number of frames without a match.
pub struct Tracker {
    tracked: Vec<TrackedTangible>,
    ema_alpha: f64,
    max_center_distance: f64,
    max_frames_without_match: u32,
}

impl Tracker {
    /// Creates a tracker.
    ///
    /// `ema_alpha` is the smoothing factor (0..1), `max_center_distance` the
    /// association radius in mm, `max_frames_without_match` the expiry
    /// hysteresis in frames.
    pub fn new(ema_alpha: f64, max_center_distance: f64, max_frames_without_match: u32) -> Self {
        Self {
            tracked: Vec::new(),
            ema_alpha,
            max_center_distance,
            max_frames_without_match,
        }
    }

    /// Processes the matches of a new frame: associate, update, create,
    /// expire.
    pub fn update(&mut self, matches: &[PatternMatch]) {
        for tangible in &mut self.tracked {
            tangible.frames_since_match += 1;
        }

        let mut used_matches = vec![false; matches.len()];
        let mut used_tracked = vec![false; self.tracked.len()];

        // Greedy nearest-neighbor association (same pattern_id only).
        for (match_index, pattern_match) in matches.iter().enumerate() {
            let mut best_ti: Option<usize> = None;
            let mut best_dist = self.max_center_distance;
            for (ti, tangible) in self.tracked.iter().enumerate() {
                if used_tracked[ti] || tangible.pattern_id != pattern_match.pattern_id {
                    continue;
                }
                let dist = tangible.center.distance(&pattern_match.center);
                if dist < best_dist {
                    best_dist = dist;
                    best_ti = Some(ti);
                }
            }
            if let Some(ti) = best_ti {
                used_matches[match_index] = true;
                used_tracked[ti] = true;
                let tangible = &mut self.tracked[ti];
                tangible.center = Vec2::new(
                    self.ema_alpha * pattern_match.center.x
                        + (1.0 - self.ema_alpha) * tangible.center.x,
                    self.ema_alpha * pattern_match.center.y
                        + (1.0 - self.ema_alpha) * tangible.center.y,
                );
                tangible.rotation_rad = RotationRad::new(ema_angle(
                    tangible.rotation_rad.as_f64(),
                    pattern_match.rotation_rad.as_f64(),
                    self.ema_alpha,
                ));
                tangible.frames_since_match = 0;
            }
        }

        // Create new tracked tangibles for unmatched matches.
        for (match_index, pattern_match) in matches.iter().enumerate() {
            if !used_matches[match_index] {
                self.tracked.push(TrackedTangible {
                    pattern_id: pattern_match.pattern_id,
                    pattern_name: pattern_match.pattern_name.clone(),
                    center: pattern_match.center,
                    rotation_rad: pattern_match.rotation_rad,
                    frames_since_match: 0,
                });
            }
        }

        // Remove stale tracked tangibles.
        self.tracked
            .retain(|t| t.frames_since_match <= self.max_frames_without_match);
    }

    /// All currently tracked tangibles.
    pub fn active(&self) -> &[TrackedTangible] {
        &self.tracked
    }

    /// Drops all tracked tangibles.
    pub fn clear(&mut self) {
        self.tracked.clear();
    }
}

/// EMA for angles with wraparound handling.
fn ema_angle(old: f64, new: f64, alpha: f64) -> f64 {
    let mut diff = new - old;
    while diff > std::f64::consts::PI {
        diff -= 2.0 * std::f64::consts::PI;
    }
    while diff < -std::f64::consts::PI {
        diff += 2.0 * std::f64::consts::PI;
    }
    old + alpha * diff
}

#[cfg(test)]
mod tests {
    use super::Tracker;
    use super::ema_angle;
    use tangible_objects_model::PatternId;
    use tangible_objects_model::PatternMatch;
    use tangible_objects_model::RotationRad;
    use tangible_objects_model::Vec2;

    fn pattern_match(id: u32, x: f64, y: f64) -> PatternMatch {
        PatternMatch {
            pattern_id: PatternId::new(id),
            pattern_name: format!("fig-{id}"),
            center: Vec2::new(x, y),
            rotation_rad: RotationRad::new(0.0),
            touch_indices: vec![0, 1, 2],
        }
    }

    #[test]
    fn new_match_creates_tracked_tangible() {
        let mut tracker = Tracker::new(0.4, 200.0, 30);
        tracker.update(&[pattern_match(1, 10.0, 20.0)]);
        assert_eq!(tracker.active().len(), 1);
        assert_eq!(tracker.active()[0].pattern_id, PatternId::new(1));
    }

    #[test]
    fn nearby_match_associates_and_smooths() {
        let mut tracker = Tracker::new(0.5, 200.0, 30);
        tracker.update(&[pattern_match(1, 10.0, 20.0)]);
        tracker.update(&[pattern_match(1, 20.0, 20.0)]);
        // EMA(0.5) of 10 and 20 → 15.
        assert!((tracker.active()[0].center.x - 15.0).abs() < 1e-9);
    }

    #[test]
    fn distant_match_creates_second_tangible() {
        let mut tracker = Tracker::new(0.4, 200.0, 30);
        tracker.update(&[pattern_match(1, 10.0, 20.0)]);
        tracker.update(&[pattern_match(1, 500.0, 500.0)]);
        assert_eq!(tracker.active().len(), 2);
    }

    #[test]
    fn tangible_expires_after_max_frames() {
        let mut tracker = Tracker::new(0.4, 200.0, 2);
        tracker.update(&[pattern_match(1, 10.0, 20.0)]);
        tracker.update(&[]);
        tracker.update(&[]);
        tracker.update(&[]);
        assert!(tracker.active().is_empty());
    }

    #[test]
    fn ema_angle_wraps_around() {
        // Crossing the -PI/+PI boundary takes the short path: 3.0 -> ~PI,
        // not down through 0.
        let smoothed = ema_angle(3.0, -3.0, 0.5);
        assert!(smoothed > 3.0);
        assert!(smoothed <= std::f64::consts::PI + 1e-9);
    }
}
