//! `RegisterFieldSetInput` - the `registerFieldSet` input object, a
//! wire DTO of [`FieldSet`].

use tangible_objects_model::Field;
use tangible_objects_model::FieldSet;
use tangible_objects_model::FieldSetName;
use tangible_objects_model::FrameCount;
use tangible_objects_model::WindowAnchor;

use super::field_input::FieldInput;
use super::window_input::WindowInput;

/// Input for `registerFieldSet`.
#[derive(async_graphql::InputObject)]
pub(crate) struct RegisterFieldSetInput {
    /// Set name (namespace for its fields).
    name: FieldSetName,
    /// Debounce stability in frames; `null` = default 2.
    stability: Option<FrameCount>,
    /// Optional window anchor.
    window: Option<WindowInput>,
    /// Whether contacts inside these fields are suppressed from the OS;
    /// `null`/`false` = occupancy is informational only.
    suppress: Option<bool>,
    /// The cells.
    fields: Vec<FieldInput>,
}

impl From<&RegisterFieldSetInput> for FieldSet {
    /// Projects the wire input into the domain field set.
    fn from(input: &RegisterFieldSetInput) -> Self {
        Self {
            name: input.name.clone(),
            window: input.window.as_ref().map(WindowAnchor::from),
            stability: input
                .stability
                .unwrap_or_else(tangible_objects_model::default_stability),
            suppress: input.suppress.unwrap_or(false),
            fields: input.fields.iter().map(Field::from).collect(),
        }
    }
}
