use std::path::PathBuf;

use async_graphql::Context;
use async_graphql::Error;
use async_graphql::Object;
use async_graphql::Result;
use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::RecordLabel;
use tangible_objects_model::Registration;
use tangible_objects_runtime::ControlCommand;
use tangible_objects_runtime::EngineHandle;

/// Control operations forwarded to the daemon's device loop.
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Switches the filter proxy's suppression policy
    /// (`off`/`immediate`/`gated`/`guarded`/`tangiblesOnly`); `stability`
    /// overrides the consecutive-matched-frames gate, `null` keeps it.
    async fn set_filter_mode(
        &self,
        ctx: &Context<'_>,
        mode: FilterMode,
        stability: Option<FrameCount>,
    ) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::SetFilterMode { mode, stability })
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Starts a training session that captures a new profile with the given
    /// name.
    async fn train_profile(&self, ctx: &Context<'_>, name: String) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::TrainProfile(name))
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Starts a recording session writing `<base_path>.jsonl` +
    /// `<base_path>.evtest`, optionally labeled.
    async fn start_recording(
        &self,
        ctx: &Context<'_>,
        base_path: String,
        label: Option<String>,
    ) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::StartRecording {
                base_path: PathBuf::from(base_path),
                label: label.map(RecordLabel::new),
            })
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Stops the active recording session.
    async fn stop_recording(&self, ctx: &Context<'_>) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::StopRecording)
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Changes the ground-truth label applied to subsequent recorded frames;
    /// `null` clears it.
    async fn set_record_label(&self, ctx: &Context<'_>, label: Option<String>) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::SetRecordLabel(label.map(RecordLabel::new)))
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Replaces the missing-pin completion configuration. Takes a full
    /// config object - read the current one via the `completion` query,
    /// modify, write back (mutation applies from the next frame on).
    async fn set_completion(
        &self,
        ctx: &Context<'_>,
        config: tangible_objects_model::CompletionConfig,
    ) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::SetCompletion(config))
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Resets the session metric counters - call between automated
    /// test runs to isolate each series.
    async fn reset_metrics(&self, ctx: &Context<'_>) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::ResetMetrics)
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Shows a user-facing message in the GUI (test-series steering);
    /// an empty string clears it.
    async fn show_message(&self, ctx: &Context<'_>, text: String) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::ShowMessage(text))
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Registers a figure profile at runtime (DYNAMIC_PROFILES.md).
    /// Queued to the device loop - fingerprint collisions are rejected
    /// there; verify via the `profiles` query.
    async fn register_pattern(
        &self,
        ctx: &Context<'_>,
        input: RegisterPatternInput,
        owner: Option<String>,
        lifetime: Option<tangible_objects_model::Lifetime>,
    ) -> Result<bool> {
        let points = flat_points(&input.points)?;
        let tolerance = tangible_objects_model::PatternTolerance::new(input.tolerance_mm);
        let mut profile =
            tangible_objects_model::ProfileFile::from_points(&input.name, &points, tolerance);
        profile.spawn_on_partial = input.spawn_on_partial.unwrap_or(false);
        profile.stage = input.stage.clone();
        if let Some(stability) = input.spawn_stability {
            profile.spawn_stability = tangible_objects_model::FrameCount::new(stability as u64);
        }
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::Register {
                registration: Box::new(Registration::Pattern(profile)),
                owner: owner.map(Owner::new),
                lifetime: lifetime.unwrap_or_default(),
            })
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Replaces a registered pattern's geometry/name while keeping its
    /// ID. Applies to new matches only.
    async fn update_profile(
        &self,
        ctx: &Context<'_>,
        id: PatternId,
        input: RegisterPatternInput,
    ) -> Result<bool> {
        let points = flat_points(&input.points)?;
        let tolerance = tangible_objects_model::PatternTolerance::new(input.tolerance_mm);
        let mut profile =
            tangible_objects_model::ProfileFile::from_points(&input.name, &points, tolerance);
        profile.spawn_on_partial = input.spawn_on_partial.unwrap_or(false);
        profile.stage = input.stage.clone();
        if let Some(stability) = input.spawn_stability {
            profile.spawn_stability = tangible_objects_model::FrameCount::new(stability as u64);
        }
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::UpdateProfile {
                id,
                profile: Box::new(profile),
            })
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Removes a dynamically registered pattern by ID.
    async fn unregister_pattern(&self, ctx: &Context<'_>, id: PatternId) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::UnregisterPattern { id })
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Removes every pattern and field set owned by `owner`.
    async fn unregister_owner(&self, ctx: &Context<'_>, owner: String) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::UnregisterOwner {
                owner: Owner::new(owner),
            })
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Registers an occupancy field set at runtime.
    async fn register_field_set(
        &self,
        ctx: &Context<'_>,
        input: RegisterFieldSetInput,
        owner: Option<String>,
        lifetime: Option<tangible_objects_model::Lifetime>,
    ) -> Result<bool> {
        let set = input.to_field_set()?;
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::Register {
                registration: Box::new(Registration::FieldSet(set)),
                owner: owner.map(Owner::new),
                lifetime: lifetime.unwrap_or_default(),
            })
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Removes a field set by name.
    async fn unregister_field_set(&self, ctx: &Context<'_>, name: String) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::UnregisterFieldSet { name })
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }
}

/// Input for `registerPattern`: a figure defined by its pin positions.
#[derive(async_graphql::InputObject)]
struct RegisterPatternInput {
    /// Figure name.
    name: String,
    /// Flat pin coordinates `[x0, y0, x1, y1, ...]` in millimeters.
    points: Vec<f64>,
    /// Scalar matching tolerance in millimeters.
    tolerance_mm: f64,
    /// Whether a partial match may spawn this figure
    /// (SPAWN_AS_PARTIAL.md); `null`/`false` = exact-only spawns.
    spawn_on_partial: Option<bool>,
    /// Frames a partial spawn must hold before confirmation;
    /// `null` = 15.
    spawn_stability: Option<i32>,
    /// Pipeline stage this pattern suppresses through
    /// (EVENT_PIPELINE.md); `null` = the general stage.
    stage: Option<tangible_objects_model::StageName>,
}

/// Input for `registerFieldSet`.
#[derive(async_graphql::InputObject)]
struct RegisterFieldSetInput {
    /// Set name (namespace for its fields).
    name: String,
    /// Debounce stability in frames; `null` = default 2.
    stability: Option<i32>,
    /// Optional window anchor.
    window: Option<WindowInput>,
    /// Whether contacts inside these fields are suppressed from the OS;
    /// `null`/`false` = occupancy is informational only.
    suppress: Option<bool>,
    /// The cells.
    fields: Vec<FieldInput>,
}

impl RegisterFieldSetInput {
    fn to_field_set(&self) -> Result<tangible_objects_model::FieldSet> {
        use tangible_objects_model::Field;
        use tangible_objects_model::FieldSet;
        use tangible_objects_model::Millimeters;
        use tangible_objects_model::Region;

        let set = FieldSet {
            name: self.name.clone().into(),
            window: self
                .window
                .as_ref()
                .map(|w| tangible_objects_model::WindowAnchor {
                    x_offset: Millimeters::new(w.x_offset),
                    y_offset: Millimeters::new(w.y_offset),
                    width: w.width.map(Millimeters::new),
                    height: w.height.map(Millimeters::new),
                }),
            stability: self
                .stability
                .map(|s| tangible_objects_model::FrameCount::new(s as u64))
                .unwrap_or_else(tangible_objects_model::default_stability),
            suppress: self.suppress.unwrap_or(false),
            fields: self
                .fields
                .iter()
                .map(|f| Field {
                    name: f.name.clone().into(),
                    region: Region {
                        x_min: Millimeters::new(f.x_min),
                        y_min: Millimeters::new(f.y_min),
                        x_max: Millimeters::new(f.x_max),
                        y_max: Millimeters::new(f.y_max),
                    },
                })
                .collect(),
        };
        Ok(set)
    }
}

/// Window anchor for a field set.
#[derive(async_graphql::InputObject)]
struct WindowInput {
    /// Window's left edge in screen mm.
    x_offset: f64,
    /// Window's top edge in screen mm.
    y_offset: f64,
    /// Window width for normalized field coordinates.
    width: Option<f64>,
    /// Window height for normalized field coordinates.
    height: Option<f64>,
}

/// One occupancy cell.
#[derive(async_graphql::InputObject)]
struct FieldInput {
    /// Cell name.
    name: String,
    /// Bounding box in mm (or 0..1 normalized when the window carries
    /// width/height).
    x_min: f64,
    /// Lower y bound.
    y_min: f64,
    /// Upper x bound.
    x_max: f64,
    /// Upper y bound.
    y_max: f64,
}

/// Parses a flat `[x0, y0, x1, y1, ...]` list into [`Vec2`]s.
fn flat_points(
    flat: &[f64],
) -> Result<Vec<tangible_objects_model::Vec2<tangible_objects_model::Millimeters>>> {
    use tangible_objects_model::Vec2;

    if !flat.len().is_multiple_of(2) || flat.is_empty() {
        return Err(Error::new(
            "points must be a non-empty even-length flat list",
        ));
    }
    Ok(flat
        .chunks_exact(2)
        .map(|pair| Vec2::new(pair[0], pair[1]))
        .collect())
}
