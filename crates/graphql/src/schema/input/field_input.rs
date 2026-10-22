//! `FieldInput` - one occupancy cell of a field set, a wire DTO of
//! [`Field`].

use tangible_objects_model::Field;
use tangible_objects_model::FieldName;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Region;

/// One occupancy cell.
#[derive(async_graphql::InputObject)]
pub(crate) struct FieldInput {
    /// Cell name.
    name: FieldName,
    /// Bounding box in mm (or 0..1 normalized when the window carries
    /// width/height).
    x_min: Millimeters,
    /// Lower y bound.
    y_min: Millimeters,
    /// Upper x bound.
    x_max: Millimeters,
    /// Upper y bound.
    y_max: Millimeters,
}

impl From<&FieldInput> for Field {
    /// Projects the wire cell into the domain field.
    fn from(f: &FieldInput) -> Self {
        Self {
            name: f.name.clone(),
            region: Region {
                x_min: f.x_min,
                y_min: f.y_min,
                x_max: f.x_max,
                y_max: f.y_max,
            },
        }
    }
}
