//! `WindowInput` - the cynic input object for the field set's
//! window anchor, a wire DTO of
//! [`tangible_objects_model::WindowAnchor`].

use cynic::InputObject;
use tangible_objects_model::Millimeters;

/// The `WindowInput` input object - wire DTO of
/// [`tangible_objects_model::WindowAnchor`].
#[derive(InputObject)]
#[cynic(
    graphql_type = "WindowInput",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    rename_all = "camelCase"
)]
pub(crate) struct WindowInput {
    /// Window's left edge in screen mm.
    pub x_offset: Millimeters,
    /// Window's top edge in screen mm.
    pub y_offset: Millimeters,
    /// Window width for normalized field coordinates.
    pub width: Option<Millimeters>,
    /// Window height for normalized field coordinates.
    pub height: Option<Millimeters>,
}
