//! `setFilterMode` mutation - the pipeline's suppression policy.

use serde_json::json;
use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::StageName;

use crate::ClientError;
use crate::GraphQLClient;
use crate::client::gql_enum;

impl GraphQLClient {
    /// Switches the suppression policy (`setFilterMode`); `stage`
    /// `Some` scopes to that stage, `None` broadcasts.
    pub async fn set_filter_mode(
        &self,
        mode: FilterMode,
        stability: Option<FrameCount>,
        stage: Option<StageName>,
    ) -> Result<bool, ClientError> {
        self.call(
            "setFilterMode",
            "mutation($mode: FilterMode!, $stability: Int, $stage: StageName) { setFilterMode(mode: $mode, stability: $stability, stage: $stage) }",
            json!({
                "mode": gql_enum(&mode),
                "stability": stability.map(|c| c.as_u64()),
                "stage": stage.map(|s| s.as_str().to_string()),
            }),
        )
        .await
    }
}
