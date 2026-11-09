//! `dead_zones()`/`dead_zone_filtered()` methods of
//! [`GraphQLClient`].

use tangible_objects_model::DeadZone;

use crate::ClientError;
use crate::GraphQLClient;
use crate::queries::dead_zones::query::DeadZoneFilteredQuery;
use crate::queries::dead_zones::query::DeadZonesQuery;

impl GraphQLClient {
    /// Registered dead zones - blacklisted surface spots (`deadZones`
    /// query).
    pub async fn dead_zones(&self) -> Result<Vec<DeadZone>, ClientError> {
        self.call_query::<DeadZonesQuery, _, _>("deadZones", ())
            .await
    }

    /// Contacts dropped by dead-zone filtering on the last processed
    /// frame (`deadZoneFiltered` query).
    pub async fn dead_zone_filtered(&self) -> Result<usize, ClientError> {
        let filtered: i32 = self
            .call_query::<DeadZoneFilteredQuery, _, _>("deadZoneFiltered", ())
            .await?;
        Ok(usize::try_from(filtered).unwrap_or_default())
    }
}
