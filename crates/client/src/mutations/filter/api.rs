//! `set_filter_mode()` method of [`GraphQLClient`].

use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::StageName;

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::filter::mutation::SetFilterMode;
use crate::mutations::filter::variables::SetFilterModeVariables;

impl GraphQLClient {
    /// Switches the suppression policy (`setFilterMode`); `stage`
    /// `Some` scopes to that stage, `None` broadcasts.
    pub async fn set_filter_mode(
        &self,
        mode: FilterMode,
        stability: Option<FrameCount>,
        stage: Option<StageName>,
    ) -> Result<bool, ClientError> {
        self.call_mutation::<SetFilterMode, _, _>(
            "setFilterMode",
            SetFilterModeVariables {
                mode,
                stability,
                stage,
            },
        )
        .await
    }
}
