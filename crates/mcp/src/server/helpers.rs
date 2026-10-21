//! Shared helpers for tool and resource impls - serialization
//! shorthands and the `tangible://` resource descriptor builder.

use rmcp::ErrorData;
use rmcp::model::Resource;
use tangible_objects_model::Lifetime;

/// Serializes a value to pretty JSON for tool/resource output.
pub(crate) fn json_pretty<T: serde::Serialize + ?Sized>(value: &T) -> Result<String, ErrorData> {
    serde_json::to_string_pretty(value)
        .map_err(|err| ErrorData::internal_error(format!("serialization failed: {err}"), None))
}

/// Parses the optional `lifetime` tool argument (`session`/`persistent`).
pub(crate) fn parse_lifetime(lifetime: &Option<String>) -> Result<Lifetime, ErrorData> {
    lifetime
        .as_deref()
        .map(str::parse)
        .transpose()
        .map(|parsed| parsed.unwrap_or_default())
        .map_err(|_| ErrorData::invalid_params("lifetime must be 'session' or 'persistent'", None))
}

/// Describes one `tangible://` resource in `resources/list`.
pub(crate) fn resource(uri: &str, name: &str, description: &str) -> Resource {
    Resource::new(uri, name)
        .with_description(description)
        .with_mime_type("application/json")
}
