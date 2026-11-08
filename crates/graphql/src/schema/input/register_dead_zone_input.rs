//! `RegisterDeadZoneInput` - the `registerDeadZone` input object, a
//! wire DTO of [`DeadZone`].

use tangible_objects_model::DEFAULT_DEAD_ZONE_RADIUS;
use tangible_objects_model::DeadZone;
use tangible_objects_model::DeadZoneName;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Vec2;

/// Input for `registerDeadZone`.
#[derive(async_graphql::InputObject)]
pub(crate) struct RegisterDeadZoneInput {
    /// Zone name (identity for unregister and the runtime store).
    name: DeadZoneName,
    /// Center x position on the surface in millimeters.
    x: Millimeters,
    /// Center y position on the surface in millimeters.
    y: Millimeters,
    /// Suppression radius in millimeters; `null` = 10 mm.
    radius: Option<Millimeters>,
}

impl From<&RegisterDeadZoneInput> for DeadZone {
    /// Projects the wire input into the domain dead zone.
    fn from(input: &RegisterDeadZoneInput) -> Self {
        Self {
            name: input.name.clone(),
            center: Vec2::from_mm(input.x, input.y),
            radius: input.radius.unwrap_or(DEFAULT_DEAD_ZONE_RADIUS),
        }
    }
}
