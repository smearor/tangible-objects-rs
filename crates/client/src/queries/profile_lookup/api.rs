//! `lookup_profile()` method of [`GraphQLClient`].

use tangible_objects_model::ProfileFile;
use tangible_objects_model::ProfileLookup;

use crate::ClientError;
use crate::GraphQLClient;
use crate::queries::profile_lookup::query::ProfileLookupQuery;
use crate::queries::profile_lookup::variables::ProfileLookupVariables;

impl GraphQLClient {
    /// Compares `profile` against the daemon's registered patterns
    /// (`profileLookup`): `matching` carries the colliding pattern -
    /// its owner and whether its fingerprint is identical to the
    /// candidate's - `None` means `register_pattern` would not collide.
    pub async fn lookup_profile(
        &self,
        profile: &ProfileFile,
    ) -> Result<ProfileLookup, ClientError> {
        self.call_query::<ProfileLookupQuery, _, _>(
            "profileLookup",
            ProfileLookupVariables {
                input: profile.into(),
            },
        )
        .await
    }
}
