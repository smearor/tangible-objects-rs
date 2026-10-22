//! `RegisterFieldSetInput` - the cynic input object for
//! `registerFieldSet`, a wire DTO of [`FieldSet`].

use cynic::InputObject;
use tangible_objects_model::FieldSet;
use tangible_objects_model::FieldSetName;
use tangible_objects_model::FrameCount;

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
    name: FieldSetName,
    /// Debounce stability in frames.
    stability: Option<FrameCount>,
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
            name: set.name.clone(),
            stability: Some(set.stability),
            window: set.window.as_ref().map(|w| WindowInput {
                x_offset: w.x_offset,
                y_offset: w.y_offset,
                width: w.width,
                height: w.height,
            }),
            suppress: Some(set.suppress),
            fields: set
                .fields
                .iter()
                .map(|f| FieldInput {
                    name: f.name.clone(),
                    x_min: f.region.x_min,
                    y_min: f.region.y_min,
                    x_max: f.region.x_max,
                    y_max: f.region.y_max,
                })
                .collect(),
        }
    }
}
