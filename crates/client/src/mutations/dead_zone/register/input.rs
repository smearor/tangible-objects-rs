//! `RegisterDeadZoneInput` - the cynic input object for
//! `registerDeadZone`, a wire DTO of [`DeadZone`].

use cynic::InputObject;
use smearor_commons_geometry::Millimeters;
use tangible_objects_model::DeadZone;
use tangible_objects_model::DeadZoneName;

/// The `RegisterDeadZoneInput` input object - wire DTO of
/// [`DeadZone`].
#[derive(InputObject)]
#[cynic(
    graphql_type = "RegisterDeadZoneInput",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    rename_all = "camelCase"
)]
pub(crate) struct RegisterDeadZoneInput {
    /// Zone name (identity for unregister and the runtime store).
    name: DeadZoneName,
    /// Center x position on the surface in millimeters.
    x: Millimeters,
    /// Center y position on the surface in millimeters.
    y: Millimeters,
    /// Suppression radius in millimeters; `None` = daemon default.
    radius: Option<Millimeters>,
}

impl From<&DeadZone> for RegisterDeadZoneInput {
    /// Projects the dead zone into the wire input shape.
    fn from(zone: &DeadZone) -> Self {
        Self {
            name: zone.name.clone(),
            x: Millimeters::new(zone.center.x),
            y: Millimeters::new(zone.center.y),
            radius: Some(zone.radius),
        }
    }
}
