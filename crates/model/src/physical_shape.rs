//! `PhysicalShape` - the declared physical outline of a tangible
//! (`[shape]` section in the profile TOML), rendered by the GUI in
//! `PhysicalGeometry`/`TouchPointsAndPhysicalGeometry` render modes.

use serde::Deserialize;
use serde::Serialize;

use crate::Millimeters;
use crate::Vec2;

/// The physical outline of a tangible's body in the pattern's frame
/// (millimeters relative to the tangible center, pattern
/// orientation - the GUI rotates it by the match's `rotation_rad`).
///
/// Serialized with a `type` tag, e.g.:
///
/// ```toml
/// [shape]
/// type = "polygon"
/// points = [[-30.0, -20.0], [30.0, -20.0], [30.0, 20.0], [-30.0, 20.0]]
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum PhysicalShape {
    /// Filled/stroked circle around the tangible center.
    Circle {
        /// Circle radius in millimeters.
        radius: Millimeters,
    },
    /// Rectangle in the pattern's frame - rotates with the tangible.
    Rectangle {
        /// Half the rectangle width in millimeters.
        half_width: Millimeters,
        /// Half the rectangle height in millimeters.
        half_height: Millimeters,
    },
    /// Ellipse in the pattern's frame - rotates with the tangible.
    Ellipse {
        /// Half the ellipse width in millimeters.
        half_width: Millimeters,
        /// Half the ellipse height in millimeters.
        half_height: Millimeters,
    },
    /// Arbitrary closed outline - e.g. a body with a hole approximated
    /// as a polygon; points in millimeters relative to the center.
    Polygon {
        /// Polygon vertices in pattern-space millimeters.
        #[serde(with = "crate::serde_points")]
        points: Vec<Vec2<Millimeters>>,
    },
}
