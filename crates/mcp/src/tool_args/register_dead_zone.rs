//! `RegisterDeadZoneArgs` - arguments for the `register_dead_zone`
//! tool.

use schemars::JsonSchema;
use serde::Deserialize;
use smearor_commons_geometry::Millimeters;
use tangible_objects_model::DeadZoneName;
use tangible_objects_model::Owner;

/// Arguments for the `register_dead_zone` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RegisterDeadZoneArgs {
    /// Zone name - the identity used for `unregister_dead_zone` and
    /// the runtime store.
    pub name: DeadZoneName,
    /// Center x position in millimeters.
    pub x: Millimeters,
    /// Center y position in millimeters.
    pub y: Millimeters,
    /// Suppression radius in millimeters; `null` = 10 mm.
    pub radius: Option<Millimeters>,
    /// Optional owner/application name.
    pub owner: Option<Owner>,
    /// `session` (default) or `persistent` (written to the runtime
    /// store). String + parse on purpose: `Lifetime` enum serde flips
    /// with the `cynic` feature.
    pub lifetime: Option<String>,
}
