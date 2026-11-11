//! `RegionInput` - a bounding-box wire DTO of [`Region`], shared by
//! spell-arming zones.

use tangible_objects_model::Millimeters;
use tangible_objects_model::Region;

/// Input for a bounding-box region in millimeters.
#[derive(async_graphql::InputObject)]
pub(crate) struct RegionInput {
    /// Lower x bound.
    x_min: Millimeters,
    /// Lower y bound.
    y_min: Millimeters,
    /// Upper x bound.
    x_max: Millimeters,
    /// Upper y bound.
    y_max: Millimeters,
}

impl From<&RegionInput> for Region {
    /// Projects the wire input into the domain region.
    fn from(input: &RegionInput) -> Self {
        Self {
            x_min: input.x_min,
            y_min: input.y_min,
            x_max: input.x_max,
            y_max: input.y_max,
        }
    }
}
