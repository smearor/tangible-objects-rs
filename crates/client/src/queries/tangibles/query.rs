//! `TangiblesQuery` - the cynic fragment for the `tangibles` query.

use cynic::QueryFragment;
use tangible_objects_model::TrackedTangible;

/// Root-field wrapper for the `tangibles` query - cynic derives the
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
pub(crate) struct TangiblesQuery {
    /// `tangibles` field.
    tangibles: Vec<TrackedTangible>,
}

#[cfg(test)]
mod tests {
    use super::TangiblesQuery;

    /// The cynic-derived document selects the wire fields of
    /// `TrackedTangible` (the bookkeeping fields stay unselected).
    #[test]
    fn tangibles_document() {
        let query = cynic::Operation::<TangiblesQuery>::query(())
            .query
            .to_string();
        for field in [
            "tangibles",
            "id",
            "patternId",
            "patternName",
            "center",
            "rotationRad",
            "pending",
            "x",
            "y",
        ] {
            assert!(query.contains(field), "missing {field} in: {query}");
        }
    }
}
