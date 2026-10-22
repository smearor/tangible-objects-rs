//! `train_profile()` method of [`GraphQLClient`].

use tangible_objects_model::PatternName;

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::training::mutation::TrainProfile;
use crate::mutations::training::variables::TrainProfileVariables;

impl GraphQLClient {
    /// Asks the device loop to record a new profile (`trainProfile`).
    pub async fn train_profile(&self, name: &PatternName) -> Result<bool, ClientError> {
        self.call_mutation::<TrainProfile, _, _>(
            "trainProfile",
            TrainProfileVariables { name: name.clone() },
        )
        .await
    }
}
