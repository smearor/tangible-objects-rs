//! `setCompletion` mutation - the missing-pin completion
//! configuration.

use serde_json::Value;
use serde_json::json;
use tangible_objects_model::CompletionConfig;

use crate::ClientError;
use crate::GraphQLClient;
use crate::client::camelize;

impl GraphQLClient {
    /// Applies a completion configuration (`setCompletion`) - the
    /// `CompletionConfig`'s snake_case serde keys are camelized into
    /// the InputObject's field names, enum values become SCREAMING_SNAKE.
    pub async fn set_completion(&self, config: &CompletionConfig) -> Result<bool, ClientError> {
        let mut input = camelize(serde_json::to_value(config).unwrap_or_default());
        if let Some(mode) = input.get("mode").and_then(Value::as_str).map(str::to_owned) {
            input["mode"] = json!(mode.replace('-', "_").to_uppercase());
        }
        self.call(
            "setCompletion",
            "mutation($config: CompletionInput!) { setCompletion(config: $config) }",
            json!({ "config": input }),
        )
        .await
    }
}
