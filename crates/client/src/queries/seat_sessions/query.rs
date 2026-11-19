//! `SeatSessionsQuery` - the cynic fragment for the `seatSessions` query.

use cynic::QueryFragment;
use cynic::QueryVariables;
use tangible_objects_model::SeatSession;
use tangible_objects_model::SeatSessionName;

/// Variables of `seatSessions` - the optional session filter.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct SeatSessionsVariables {
    /// Narrow to one session - `None` lists every claimed session.
    pub session: Option<SeatSessionName>,
}

/// Root-field wrapper for the `seatSessions` query - cynic derives the
/// document and checks every selected field against `schema.graphql`
/// at compile time. The fields are selection markers only - the
/// response decodes through the model's `Deserialize`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "QueryRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    variables = "SeatSessionsVariables"
)]
pub(crate) struct SeatSessionsQuery {
    /// `seatSessions` field.
    #[arguments(session: $session)]
    seat_sessions: Vec<SeatSession>,
}

#[cfg(test)]
mod tests {
    use super::SeatSessionsQuery;

    /// The cynic-derived document selects the session fields and the
    /// nested `TableTopSeat` roster.
    #[test]
    fn seat_sessions_document() {
        let query = cynic::Operation::<SeatSessionsQuery>::query(())
            .query
            .to_string();
        for field in [
            "seatSessions",
            "seats",
            "id",
            "name",
            "owner",
            "marker",
            "tangible",
            "pose",
            "state",
        ] {
            assert!(query.contains(field), "missing {field} in: {query}");
        }
    }
}
