//! Field-set mutations - `registerFieldSet`/`unregisterFieldSet`.

use serde_json::Value;
use serde_json::json;
use tangible_objects_model::FieldSet;

use crate::ClientError;
use crate::GraphQLClient;

impl GraphQLClient {
    /// Registers an occupancy field set (`registerFieldSet`) - the
    /// `FieldSet`'s region/members flatten into the InputObject shape.
    pub async fn register_field_set(&self, set: &FieldSet) -> Result<bool, ClientError> {
        let fields: Vec<Value> = set
            .fields
            .iter()
            .map(|f| {
                json!({
                    "name": f.name.as_str(),
                    "xMin": f.region.x_min.as_f64(),
                    "yMin": f.region.y_min.as_f64(),
                    "xMax": f.region.x_max.as_f64(),
                    "yMax": f.region.y_max.as_f64(),
                })
            })
            .collect();
        self.call(
            "registerFieldSet",
            "mutation($set: RegisterFieldSetInput!) { registerFieldSet(set: $set) }",
            json!({
                "set": {
                    "name": set.name.as_str(),
                    "stability": set.stability.as_u64() as i32,
                    "window": set.window.as_ref().map(|w| json!({
                        "xOffset": w.x_offset.as_f64(),
                        "yOffset": w.y_offset.as_f64(),
                        "width": w.width.map(|v| v.as_f64()),
                        "height": w.height.map(|v| v.as_f64()),
                    })),
                    "suppress": set.suppress,
                    "fields": fields,
                },
            }),
        )
        .await
    }

    /// Removes a field set (`unregisterFieldSet`).
    pub async fn unregister_field_set(&self, name: &str) -> Result<bool, ClientError> {
        self.call(
            "unregisterFieldSet",
            "mutation($name: String!) { unregisterFieldSet(name: $name) }",
            json!({ "name": name }),
        )
        .await
    }
}
