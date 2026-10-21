//! `RegisterFieldSetInput` - the cynic input object for
//! `registerFieldSet`, a wire DTO of [`FieldSet`].

use cynic::InputObject;
use tangible_objects_model::FieldSet;

use crate::mutations::field_set::register::field_input::FieldInput;
use crate::mutations::field_set::register::window_input::WindowInput;

/// The `RegisterFieldSetInput` input object - wire DTO of
/// [`FieldSet`].
#[derive(InputObject)]
#[cynic(
    graphql_type = "RegisterFieldSetInput",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    rename_all = "camelCase"
)]
pub(crate) struct RegisterFieldSetInput {
    /// Set name (namespace for its fields).
    name: String,
    /// Debounce stability in frames.
    stability: Option<i32>,
    /// Optional window anchor.
    window: Option<WindowInput>,
    /// Whether contacts inside these fields are suppressed.
    suppress: Option<bool>,
    /// The cells.
    fields: Vec<FieldInput>,
}

impl From<&FieldSet> for RegisterFieldSetInput {
    /// Projects the field set into the wire input shape.
    fn from(set: &FieldSet) -> Self {
        Self {
            name: set.name.as_str().to_string(),
            stability: Some(set.stability.as_u64() as i32),
            window: set.window.as_ref().map(|w| WindowInput {
                x_offset: w.x_offset.as_f64(),
                y_offset: w.y_offset.as_f64(),
                width: w.width.map(|v| v.as_f64()),
                height: w.height.map(|v| v.as_f64()),
            }),
            suppress: Some(set.suppress),
            fields: set
                .fields
                .iter()
                .map(|f| FieldInput {
                    name: f.name.as_str().to_string(),
                    x_min: f.region.x_min.as_f64(),
                    y_min: f.region.y_min.as_f64(),
                    x_max: f.region.x_max.as_f64(),
                    y_max: f.region.y_max.as_f64(),
                })
                .collect(),
        }
    }
}
