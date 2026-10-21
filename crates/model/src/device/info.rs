use crate::Calibration;
use crate::RawAxisValue;
use crate::UnitsPerMm;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

/// Static information about the input device attached to this daemon -
/// the identity needed to interpret or convert its raw event stream.
///
/// Published once at session start inside [`crate::EngineSnapshot`];
/// interfaces read it via the `device_info` query (remote recording:
/// computer A learns B's touchscreen geometry before pulling artifacts).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "DeviceInfo",
        no_deserialize
    )
)]
pub struct DeviceInfo {
    /// `Input device name` of the evdev device (or the replayed file).
    pub name: Option<String>,
    /// Device units per millimeter on X (`ABS_MT_POSITION_X` resolution).
    pub x_res: Option<UnitsPerMm>,
    /// Device units per millimeter on Y (`ABS_MT_POSITION_Y` resolution).
    pub y_res: Option<UnitsPerMm>,
    /// Minimum raw X value.
    pub x_min: Option<RawAxisValue>,
    /// Maximum raw X value.
    pub x_max: Option<RawAxisValue>,
    /// Minimum raw Y value.
    pub y_min: Option<RawAxisValue>,
    /// Maximum raw Y value.
    pub y_max: Option<RawAxisValue>,
}

impl DeviceInfo {
    /// Builds the info from a device name and its calibration.
    pub fn of(name: Option<&str>, calibration: Option<&Calibration>) -> Self {
        match calibration {
            Some(cal) => Self {
                name: name.map(|s| s.to_string()),
                x_res: Some(cal.x_res),
                y_res: Some(cal.y_res),
                x_min: Some(cal.x_min),
                x_max: Some(cal.x_max),
                y_min: Some(cal.y_min),
                y_max: Some(cal.y_max),
            },
            None => Self {
                name: name.map(|s| s.to_string()),
                ..Default::default()
            },
        }
    }
}
