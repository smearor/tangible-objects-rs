//! `TouchPointsQuery` - the cynic fragment for the `touchPoints`
//! query.

use cynic::QueryFragment;
use tangible_objects_model::TouchPoint;

/// Root-field wrapper for the `touchPoints` query - cynic derives the
/// document and checks every selected field against `schema.graphql`
/// at compile time. The fields are selection markers only - the
/// response decodes through the model's `Deserialize`.
#[allow(dead_code)]
#[derive(QueryFragment)]
#[cynic(
    graphql_type = "QueryRoot",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema"
)]
pub(crate) struct TouchPointsQuery {
    /// `touchPoints` field.
    #[cynic(rename = "touchPoints")]
    touch_points: Vec<TouchPoint>,
}

#[cfg(test)]
mod tests {
    use super::TouchPointsQuery;

    /// The cynic-derived document selects every `TouchPoint` field.
    #[test]
    fn touch_points_document() {
        let query = cynic::Operation::<TouchPointsQuery>::query(())
            .query
            .to_string();
        for field in ["touchPoints", "slot", "trackingId", "pos", "x", "y"] {
            assert!(query.contains(field), "missing {field} in: {query}");
        }
    }
}
