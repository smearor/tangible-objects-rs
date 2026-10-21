//! Device query methods - `device_info`.

use crate::service::engine_service::EngineService;

/// Attached input device as a flat key-value list
/// (`name`, `x_res`, `y_res`, `x_min`, `x_max`, `y_min`, `y_max`);
/// empty values when no device/calibration is known.
pub(crate) async fn device_info(service: &EngineService) -> Vec<(String, String)> {
    let info = service.handle.snapshot().device;
    let Some(info) = info else {
        return Vec::new();
    };
    [
        ("name", info.name),
        ("x_res", info.x_res.map(|v| v.to_string())),
        ("y_res", info.y_res.map(|v| v.to_string())),
        ("x_min", info.x_min.map(|v| v.to_string())),
        ("x_max", info.x_max.map(|v| v.to_string())),
        ("y_min", info.y_min.map(|v| v.to_string())),
        ("y_max", info.y_max.map(|v| v.to_string())),
    ]
    .into_iter()
    .filter_map(|(key, value)| value.map(|v| (key.to_string(), v)))
    .collect()
}
