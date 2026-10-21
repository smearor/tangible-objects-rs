//! `RegisterPatternInput` - the cynic input object for
//! `registerPattern`/`updateProfile`, a wire DTO of [`ProfileFile`].

use tangible_objects_model::ProfileFile;
use tangible_objects_model::StageName;

/// The `RegisterPatternInput` input object - wire DTO built from a
/// [`ProfileFile`] (`points` flatten to `[x0, y0, x1, y1, ...]`).
#[derive(cynic::InputObject)]
#[cynic(
    graphql_type = "RegisterPatternInput",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    rename_all = "camelCase"
)]
pub(crate) struct RegisterPatternInput {
    /// Figure name.
    name: String,
    /// Flat pin coordinates in millimeters.
    points: Vec<f64>,
    /// Scalar matching tolerance in millimeters.
    tolerance_mm: f64,
    /// Whether a partial match may spawn this figure.
    spawn_on_partial: Option<bool>,
    /// Frames a partial spawn must hold before confirmation.
    spawn_stability: Option<i32>,
    /// Pipeline stage this pattern suppresses through.
    stage: Option<StageName>,
}

impl From<&ProfileFile> for RegisterPatternInput {
    /// Projects the profile file into the wire input shape.
    fn from(profile: &ProfileFile) -> Self {
        Self {
            name: profile.name.as_str().to_string(),
            points: profile.points.iter().flat_map(|p| [p.x, p.y]).collect(),
            tolerance_mm: profile.tolerance.as_f64(),
            spawn_on_partial: Some(profile.spawn_on_partial),
            spawn_stability: Some(profile.spawn_stability.as_u64() as i32),
            stage: profile.stage.clone(),
        }
    }
}
