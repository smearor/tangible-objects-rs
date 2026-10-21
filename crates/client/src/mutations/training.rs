//! `trainProfile` mutation - asks the device loop to capture a new
//! profile from a stabilized contact set.

use serde_json::json;

use crate::ClientError;
use crate::GraphQLClient;

impl GraphQLClient {
    /// Asks the device loop to record a new profile (`trainProfile`).
    pub async fn train_profile(&self, name: &str) -> Result<bool, ClientError> {
        self.call(
            "trainProfile",
            "mutation($name: String!) { trainProfile(name: $name) }",
            json!({ "name": name }),
        )
        .await
    }
}
