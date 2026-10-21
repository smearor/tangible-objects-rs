//! `DeviceInfoQuery` - the cynic fragment for the `deviceInfo` query.

use cynic::QueryFragment;
use tangible_objects_model::DeviceInfo;

/// Root-field wrapper for the `deviceInfo` query - cynic derives the
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
pub(crate) struct DeviceInfoQuery {
    /// `deviceInfo` field.
    #[cynic(rename = "deviceInfo")]
    device_info: Option<DeviceInfo>,
}

#[cfg(test)]
mod tests {
    use super::DeviceInfoQuery;

    /// The cynic-derived document selects every `DeviceInfo` field -
    /// compile-time field check plus this string assert as the visible
    /// proof.
    #[test]
    fn device_info_document() {
        let query = cynic::Operation::<DeviceInfoQuery>::query(())
            .query
            .to_string();
        for field in [
            "deviceInfo",
            "name",
            "xRes",
            "yRes",
            "xMin",
            "xMax",
            "yMin",
            "yMax",
        ] {
            assert!(query.contains(field), "missing {field} in: {query}");
        }
    }
}
