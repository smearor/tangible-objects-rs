//! `FieldInput` - the cynic input object for one occupancy cell.

use cynic::InputObject;
use tangible_objects_model::FieldName;
use tangible_objects_model::Millimeters;

/// The `FieldInput` input object - a flat-named occupancy cell with
/// millimeter bounds.
#[derive(InputObject)]
#[cynic(
    graphql_type = "FieldInput",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    rename_all = "camelCase"
)]
pub(crate) struct FieldInput {
    /// Cell name.
    pub name: FieldName,
    /// Bounding box in mm (or 0..1 normalized under a window).
    pub x_min: Millimeters,
    /// Lower y bound.
    pub y_min: Millimeters,
    /// Upper x bound.
    pub x_max: Millimeters,
    /// Upper y bound.
    pub y_max: Millimeters,
}
