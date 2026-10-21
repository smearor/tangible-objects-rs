//! `device_info()` method of [`GraphQLClient`].

use tangible_objects_model::DeviceInfo;

use crate::ClientError;
use crate::GraphQLClient;
use crate::queries::device::query::DeviceInfoQuery;

impl GraphQLClient {
    /// Attached input device info (`deviceInfo` query).
    pub async fn device_info(&self) -> Result<Option<DeviceInfo>, ClientError> {
        self.call_query::<DeviceInfoQuery, _, _>("deviceInfo", ())
            .await
    }
}
