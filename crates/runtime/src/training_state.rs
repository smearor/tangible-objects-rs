//! Remote profile training (`ControlCommand::TrainProfile`) - a
//! capture request living on the runtime; the device loops call
//! [`TrainingState::tick`] per frame and register the built profile.

use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternName;
use tangible_objects_model::PatternTolerance;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::Vec2;

/// Default tolerance for remotely trained profiles (matches the GUI's
/// save button).
const DEFAULT_TOLERANCE: PatternTolerance = PatternTolerance::new(3.0);

/// Consecutive equal-sized frames the contact set must hold before the
/// profile is captured (mirrors the GUI's `STABLE_THRESHOLD`).
const STABLE_THRESHOLD: u32 = 10;

/// Pending remote-training request - tracks contact-set stability over
/// frames until the point set can be captured as a [`ProfileFile`].
#[derive(Debug)]
pub struct TrainingState {
    /// Figure name for the trained profile.
    name: PatternName,
    /// Contact count of the last seen frame.
    stable_count: Option<usize>,
    /// Consecutive frames the count stayed unchanged.
    stable_frames: u32,
}

impl TrainingState {
    /// Starts a capture for `name` - the next stable point set (≥3
    /// contacts, ≥[`STABLE_THRESHOLD`] equal-sized frames) becomes the
    /// profile.
    pub fn new(name: &str) -> Self {
        Self {
            name: PatternName::new(name),
            stable_count: None,
            stable_frames: 0,
        }
    }

    /// The requested figure name.
    pub fn name(&self) -> &PatternName {
        &self.name
    }

    /// Advances the stability counter with this frame's touches;
    /// returns the captured profile once the set is stable.
    pub fn tick(&mut self, touches: &[TouchPoint]) -> Option<ProfileFile> {
        let count = touches.len();
        if self.stable_count == Some(count) && count >= 3 {
            self.stable_frames += 1;
        } else {
            self.stable_count = Some(count);
            self.stable_frames = 0;
        }
        if self.stable_frames >= STABLE_THRESHOLD {
            let points: Vec<Vec2<Millimeters>> = touches.iter().map(|t| t.pos).collect();
            return Some(ProfileFile::from_points(
                self.name.as_str(),
                &points,
                DEFAULT_TOLERANCE,
            ));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tangible_objects_model::TouchPointSlot;
    use tangible_objects_model::TrackingId;

    fn touches(n: usize) -> Vec<TouchPoint> {
        (0..n)
            .map(|i| TouchPoint {
                slot: TouchPointSlot::new(i as u32),
                tracking_id: TrackingId::new(i as i32),
                pos: Vec2::new(i as f64 * 10.0, 0.0),
            })
            .collect()
    }

    #[test]
    fn captures_after_stable_frames() {
        let mut state = TrainingState::new("fig");
        for _ in 0..STABLE_THRESHOLD {
            assert!(state.tick(&touches(4)).is_none());
        }
        assert!(state.tick(&touches(4)).is_some());
    }

    #[test]
    fn resets_on_count_change() {
        let mut state = TrainingState::new("fig");
        for _ in 0..STABLE_THRESHOLD {
            state.tick(&touches(4));
        }
        assert!(state.tick(&touches(5)).is_none());
        for _ in 0..STABLE_THRESHOLD {
            state.tick(&touches(5));
        }
        assert!(state.tick(&touches(5)).is_some());
    }

    #[test]
    fn ignores_sub_minimal_sets() {
        let mut state = TrainingState::new("fig");
        for _ in 0..STABLE_THRESHOLD + 4 {
            assert!(state.tick(&touches(2)).is_none());
        }
    }
}
