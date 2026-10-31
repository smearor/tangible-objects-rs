//! `RegisterPatternInput` - the `registerPattern`/`updateProfile`
//! input object, a figure defined by its pin positions.

use async_graphql::Error;
use async_graphql::Result;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Millimeters;
use tangible_objects_model::PatternName;
use tangible_objects_model::PatternTolerance;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::StageName;
use tangible_objects_model::Vec2;

/// Input for `registerPattern`: a figure defined by its pin positions.
#[derive(async_graphql::InputObject)]
pub(crate) struct RegisterPatternInput {
    /// Figure name.
    name: PatternName,
    /// Flat pin coordinates `[x0, y0, x1, y1, ...]` in millimeters.
    points: Vec<Millimeters>,
    /// Scalar matching tolerance in millimeters.
    tolerance_mm: PatternTolerance,
    /// Whether a partial match may spawn this figure
    /// (SPAWN_AS_PARTIAL.md); `null`/`false` = exact-only spawns.
    spawn_on_partial: Option<bool>,
    /// Frames a partial spawn must hold before confirmation;
    /// `null` = 15.
    spawn_stability: Option<FrameCount>,
    /// Pipeline stage this pattern suppresses through
    /// (EVENT_PIPELINE.md); `null` = the general stage.
    stage: Option<StageName>,
}

impl TryFrom<&RegisterPatternInput> for ProfileFile {
    type Error = Error;

    /// Projects the wire input into the domain profile file.
    fn try_from(input: &RegisterPatternInput) -> Result<Self> {
        let mut profile = ProfileFile::from_points(
            input.name.as_str(),
            &input.points_vec2()?,
            input.tolerance_mm,
        );
        profile.spawn_on_partial = input.spawn_on_partial.unwrap_or(false);
        profile.stage = input.stage.clone();
        if let Some(stability) = input.spawn_stability {
            profile.spawn_stability = stability;
        }
        Ok(profile)
    }
}

impl RegisterPatternInput {
    /// Parses the flat `[x0, y0, x1, y1, ...]` list into [`Vec2`]s.
    fn points_vec2(&self) -> Result<Vec<Vec2<Millimeters>>> {
        if !self.points.len().is_multiple_of(2) || self.points.is_empty() {
            return Err(Error::new(
                "points must be a non-empty even-length flat list",
            ));
        }
        Ok(self
            .points
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| Vec2::new(pair[0].as_f64(), pair[1].as_f64()))
            .collect())
    }
}
