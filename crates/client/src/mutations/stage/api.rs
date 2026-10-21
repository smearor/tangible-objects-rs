//! `register_stage()`/`unregister_stage()` methods of
//! [`GraphQLClient`].

use tangible_objects_model::StageName;

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::stage::register::mutation::RegisterStage;
use crate::mutations::stage::unregister::mutation::UnregisterStage;
use crate::mutations::stage::variables::StageVariables;

impl GraphQLClient {
    /// Claims or creates a tangible stage (`registerStage`).
    pub async fn register_stage(&self, name: &StageName) -> Result<bool, ClientError> {
        self.call_mutation::<RegisterStage, _, _>(
            "registerStage",
            StageVariables { name: name.clone() },
        )
        .await
    }

    /// Removes a stage (`unregisterStage`).
    pub async fn unregister_stage(&self, name: &StageName) -> Result<bool, ClientError> {
        self.call_mutation::<UnregisterStage, _, _>(
            "unregisterStage",
            StageVariables { name: name.clone() },
        )
        .await
    }
}
