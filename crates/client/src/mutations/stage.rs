//! Stage lifecycle mutations - `registerStage`/`unregisterStage`.

use serde_json::json;
use tangible_objects_model::StageName;

use crate::ClientError;
use crate::GraphQLClient;

impl GraphQLClient {
    /// Claims or creates a tangible stage (`registerStage`).
    pub async fn register_stage(&self, name: &StageName) -> Result<bool, ClientError> {
        self.call(
            "registerStage",
            "mutation($name: StageName!) { registerStage(name: $name) }",
            json!({ "name": name.as_str() }),
        )
        .await
    }

    /// Removes a stage (`unregisterStage`).
    pub async fn unregister_stage(&self, name: &StageName) -> Result<bool, ClientError> {
        self.call(
            "unregisterStage",
            "mutation($name: StageName!) { unregisterStage(name: $name) }",
            json!({ "name": name.as_str() }),
        )
        .await
    }
}
