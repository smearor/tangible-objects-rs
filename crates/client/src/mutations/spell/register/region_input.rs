//! `RegionInput` - the cynic input object for a surface rectangle,
//! a wire DTO of [`Region`].

use cynic::InputObject;
use smearor_commons_geometry::Millimeters;
use tangible_objects_model::Region;

/// The `RegionInput` input object - wire DTO of [`Region`].
#[derive(InputObject)]
#[cynic(
    graphql_type = "RegionInput",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    rename_all = "camelCase"
)]
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

impl From<&Region> for RegionInput {
    /// Projects the region into the wire input shape.
    fn from(region: &Region) -> Self {
        Self {
            x_min: region.x_min,
            y_min: region.y_min,
            x_max: region.x_max,
            y_max: region.y_max,
        }
    }
}
