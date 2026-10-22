//! `WindowInput` - the optional window anchor of a field set, a wire
//! DTO of [`WindowAnchor`].

use tangible_objects_model::Millimeters;
use tangible_objects_model::WindowAnchor;

/// Window anchor for a field set.
#[derive(async_graphql::InputObject)]
pub(crate) struct WindowInput {
    /// Window's left edge in screen mm.
    x_offset: Millimeters,
    /// Window's top edge in screen mm.
    y_offset: Millimeters,
    /// Window width for normalized field coordinates.
    width: Option<Millimeters>,
    /// Window height for normalized field coordinates.
    height: Option<Millimeters>,
}

impl From<&WindowInput> for WindowAnchor {
    /// Projects the wire input into the domain anchor.
    fn from(w: &WindowInput) -> Self {
        Self {
            x_offset: w.x_offset,
            y_offset: w.y_offset,
            width: w.width,
            height: w.height,
        }
    }
}
