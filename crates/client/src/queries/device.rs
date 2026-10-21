//! `deviceInfo` query - the daemon's input device identity and axis
//! calibration.

use serde_json::json;
use tangible_objects_model::DeviceInfo;

use crate::ClientError;
use crate::GraphQLClient;

impl GraphQLClient {
    /// Attached input device info (`deviceInfo` query).
    pub async fn device_info(&self) -> Result<Option<DeviceInfo>, ClientError> {
        self.call(
            "deviceInfo",
            "{ deviceInfo { name xRes yRes xMin xMax yMin yMax } }",
            json!({}),
        )
        .await
    }
}
