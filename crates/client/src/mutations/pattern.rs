//! Pattern lifecycle mutations - `registerPattern`, `updateProfile`,
//! `unregisterPattern` and `unregisterOwner`.

use serde_json::json;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::ProfileFile;

use crate::ClientError;
use crate::GraphQLClient;
use crate::client::gql_enum;

impl GraphQLClient {
    /// Registers a profile at runtime (`registerPattern`) - the
    /// `ProfileFile`'s geometry is flattened into the InputObject's
    /// `[x0, y0, x1, y1, ...]` form.
    pub async fn register_pattern(
        &self,
        profile: &ProfileFile,
        owner: Option<&Owner>,
        lifetime: Lifetime,
    ) -> Result<bool, ClientError> {
        let points: Vec<f64> = profile.points.iter().flat_map(|p| [p.x, p.y]).collect();
        self.call(
            "registerPattern",
            "mutation($input: RegisterPatternInput!, $owner: String, $lifetime: Lifetime) { registerPattern(input: $input, owner: $owner, lifetime: $lifetime) }",
            json!({
                "input": {
                    "name": profile.name.as_str(),
                    "points": points,
                    "toleranceMm": profile.tolerance.as_f64(),
                    "spawnOnPartial": profile.spawn_on_partial,
                    "spawnStability": profile.spawn_stability.as_u64() as i32,
                    "stage": profile.stage.as_ref().map(|s| s.as_str()),
                },
                "owner": owner.map(|o| o.as_str()),
                "lifetime": gql_enum(&lifetime),
            }),
        )
        .await
    }

    /// Replaces a registered pattern's geometry (`updateProfile`).
    pub async fn update_profile(
        &self,
        id: PatternId,
        profile: &ProfileFile,
    ) -> Result<bool, ClientError> {
        let points: Vec<f64> = profile.points.iter().flat_map(|p| [p.x, p.y]).collect();
        self.call(
            "updateProfile",
            "mutation($id: PatternId!, $input: RegisterPatternInput!) { updateProfile(id: $id, input: $input) }",
            json!({
                "id": id.as_u32(),
                "input": {
                    "name": profile.name.as_str(),
                    "points": points,
                    "toleranceMm": profile.tolerance.as_f64(),
                    "spawnOnPartial": profile.spawn_on_partial,
                    "spawnStability": profile.spawn_stability.as_u64() as i32,
                    "stage": profile.stage.as_ref().map(|s| s.as_str()),
                },
            }),
        )
        .await
    }

    /// Removes a registered pattern (`unregisterPattern`).
    pub async fn unregister_pattern(&self, id: PatternId) -> Result<bool, ClientError> {
        self.call(
            "unregisterPattern",
            "mutation($id: PatternId!) { unregisterPattern(id: $id) }",
            json!({ "id": id.as_u32() }),
        )
        .await
    }

    /// Drops every registration of `owner` (`unregisterOwner`).
    pub async fn unregister_owner(&self, owner: &Owner) -> Result<bool, ClientError> {
        self.call(
            "unregisterOwner",
            "mutation($owner: String!) { unregisterOwner(owner: $owner) }",
            json!({ "owner": owner.as_str() }),
        )
        .await
    }
}
