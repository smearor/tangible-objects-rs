//! `TangibleRenderMode` - how the GUI draws a matched tangible
//! (GUI.md's contact-point visuals vs the profile's declared physical
//! outline).

use serde::Deserialize;
use serde::Serialize;

/// Which layers the live view draws for a matched tangible.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TangibleRenderMode {
    /// Contact points only - the complete-graph lines and touch
    /// circles over the matched slots.
    TouchPoints,
    /// Only the profile's declared [`crate::PhysicalShape`] - no
    /// contact visuals (the contact data is the shape, the figure is
    /// implied).
    PhysicalGeometry,
    /// Physical shape beneath the contact visuals - the default debug
    /// view.
    #[default]
    TouchPointsAndPhysicalGeometry,
}
