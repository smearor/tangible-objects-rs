//! `Episode` - the facts a completed gesture episode carries into
//! arming evaluation.

use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;
use tangible_objects_model::HandName;
use tangible_objects_model::Region;
use tangible_objects_model::TouchPointSlot;

/// Whether an episode satisfied a spell's arming condition - the
/// `Listening -> Armed` transition's per-episode evaluation.
pub(super) struct Episode<'a> {
    /// Completed strokes of the episode with their slots - the
    /// slots identify what a matched spell consumed.
    pub(super) strokes: &'a [(TouchPointSlot, Vec<Vec2<Millimeters>>)],
    /// The episode's very first contact point - `start-spot` checks
    /// this against its region.
    pub(super) first_point: Option<Vec2<Millimeters>>,
    /// `five-finger-tap` latch active during the episode.
    pub(super) five_tap_armed: bool,
    /// The hand the tap latch identified - a `five-finger-tap` spell
    /// naming a `hand` only arms when the tap's constellation
    /// matched that handprint (per-user spells, SPELLS.md).
    pub(super) armed_hand: Option<HandName>,
}

impl Episode<'_> {
    /// Stroke points without slot identity - the matcher input.
    pub(super) fn polylines(&self) -> Vec<Vec<Vec2<Millimeters>>> {
        self.strokes
            .iter()
            .map(|(_, points)| points.clone())
            .collect()
    }

    /// `exclusive-zone` arming: every stroke point must lie inside
    /// the region - leaving the zone fails the episode.
    pub(super) fn all_points_inside(&self, region: &Region) -> bool {
        self.strokes
            .iter()
            .all(|(_, stroke)| stroke.iter().all(|point| region.contains(point)))
    }
}
