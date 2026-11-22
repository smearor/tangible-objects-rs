//! `RegisterHandProfileInput` - the cynic input object for
//! `registerHandProfile`, a wire DTO of [`HandProfile`].

use cynic::InputObject;
use smearor_commons_geometry::Millimeters;
use tangible_objects_model::HandName;
use tangible_objects_model::HandProfile;

/// The `RegisterHandProfileInput` input object - wire DTO of
/// [`HandProfile`].
#[derive(InputObject)]
#[cynic(
    graphql_type = "RegisterHandProfileInput",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    rename_all = "camelCase"
)]
pub(crate) struct RegisterHandProfileInput {
    /// Hand name (identity for unregister and the runtime store).
    name: HandName,
    /// Max per-distance deviation in millimeters; `None` = daemon
    /// default.
    tolerance: Option<Millimeters>,
    /// The recorded contact constellation - flat `[x0, y0, x1, y1,
    /// ...]` pairs in millimeters.
    points: Vec<Millimeters>,
}

impl From<&HandProfile> for RegisterHandProfileInput {
    /// Projects the handprint into the wire input shape - the
    /// `Vec2` points flatten into `[x0, y0, x1, y1, ...]` wire form.
    fn from(hand: &HandProfile) -> Self {
        Self {
            name: hand.name.clone(),
            tolerance: Some(hand.tolerance),
            points: hand
                .points
                .iter()
                .flat_map(|point| [Millimeters::new(point.x), Millimeters::new(point.y)])
                .collect(),
        }
    }
}
