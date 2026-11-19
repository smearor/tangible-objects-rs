//! `seat_sessions()` method of [`GraphQLClient`].

use tangible_objects_model::SeatSession;
use tangible_objects_model::SeatSessionName;

use crate::ClientError;
use crate::GraphQLClient;
use crate::queries::seat_sessions::query::SeatSessionsQuery;
use crate::queries::seat_sessions::query::SeatSessionsVariables;

impl GraphQLClient {
    /// Claimed seat sessions with their rosters (`seatSessions`
    /// query) - `session` narrows the listing to one session.
    pub async fn seat_sessions(
        &self,
        session: Option<&SeatSessionName>,
    ) -> Result<Vec<SeatSession>, ClientError> {
        self.call_query::<SeatSessionsQuery, _, _>(
            "seatSessions",
            SeatSessionsVariables {
                session: session.cloned(),
            },
        )
        .await
    }
}
