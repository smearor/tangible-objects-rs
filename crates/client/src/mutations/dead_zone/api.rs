//! `register_dead_zone()`/`unregister_dead_zone()` methods of
//! [`GraphQLClient`].

use tangible_objects_model::DeadZone;
use tangible_objects_model::DeadZoneName;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::RegisterDeadZoneResult;

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::dead_zone::register::mutation::RegisterDeadZone;
use crate::mutations::dead_zone::register::result::RegisterDeadZoneResultFragment;
use crate::mutations::dead_zone::register::variables::RegisterDeadZoneVariables;
use crate::mutations::dead_zone::unregister::mutation::UnregisterDeadZone;
use crate::mutations::dead_zone::unregister::variables::UnregisterDeadZoneVariables;

impl GraphQLClient {
    /// Registers a dead zone (`registerDeadZone`) - contacts inside
    /// the suppressed spot are dropped at ingest. The result union
    /// carries the typed rejection, not just a message.
    pub async fn register_dead_zone(
        &self,
        zone: &DeadZone,
        owner: Option<&Owner>,
        lifetime: Lifetime,
    ) -> Result<RegisterDeadZoneResult, ClientError> {
        let fragment: RegisterDeadZoneResultFragment = self
            .call_mutation::<RegisterDeadZone, _, _>(
                "registerDeadZone",
                RegisterDeadZoneVariables {
                    input: zone.into(),
                    owner: owner.cloned(),
                    lifetime: Some(lifetime),
                },
            )
            .await?;
        fragment.into_result()
    }

    /// Removes a dead zone (`unregisterDeadZone`).
    pub async fn unregister_dead_zone(&self, name: &DeadZoneName) -> Result<bool, ClientError> {
        self.call_mutation::<UnregisterDeadZone, _, _>(
            "unregisterDeadZone",
            UnregisterDeadZoneVariables { name: name.clone() },
        )
        .await
    }
}
