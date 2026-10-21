//! `profiles` query - loaded and registered figure profiles.

use tangible_objects_model::ProfileInfo;

use crate::ClientError;
use crate::GraphQLClient;
use crate::ProfileQuery;

impl GraphQLClient {
    /// Loaded/registered figure profiles (`profiles` query, narrowed
    /// by `query`).
    pub async fn profiles(&self, query: &ProfileQuery) -> Result<Vec<ProfileInfo>, ClientError> {
        self.call(
            "profiles",
            "query($owner: String, $staticOnly: Boolean) { profiles(owner: $owner, staticOnly: $staticOnly) { patternId name numPoints owner spawnOnPartial spawnStability stage } }",
            serde_json::to_value(query).unwrap_or_default(),
        )
        .await
    }
}
