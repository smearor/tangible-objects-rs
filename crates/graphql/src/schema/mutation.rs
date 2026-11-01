use std::path::PathBuf;

use async_graphql::Context;
use async_graphql::Error;
use async_graphql::Object;
use async_graphql::Result;
use tangible_objects_model::CompletionConfig;
use tangible_objects_model::DisplayRotation;
use tangible_objects_model::FieldSet;
use tangible_objects_model::FieldSetName;
use tangible_objects_model::FieldSetRegistered;
use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternName;
use tangible_objects_model::PatternRegistered;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::RecordLabel;
use tangible_objects_model::RegisterFieldSetResult;
use tangible_objects_model::RegisterPatternResult;
use tangible_objects_model::Registration;
use tangible_objects_model::RegistrationApplied;
use tangible_objects_model::StageName;
use tangible_objects_runtime::ControlCommand;
use tangible_objects_runtime::EngineHandle;

use crate::schema::input::RegisterFieldSetInput;
use crate::schema::input::RegisterPatternInput;

/// Control operations forwarded to the daemon's device loop.
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Switches the filter proxy's suppression policy
    /// (`off`/`immediate`/`gated`/`guarded`/`tangiblesOnly`); `stability`
    /// overrides the consecutive-matched-frames gate, `null` keeps it.
    /// `stage` narrows the switch to that scope's tangible stage,
    /// `null` broadcasts to all.
    async fn set_filter_mode(
        &self,
        ctx: &Context<'_>,
        mode: FilterMode,
        stability: Option<FrameCount>,
        stage: Option<StageName>,
    ) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::SetFilterMode {
                mode,
                stability,
                stage,
            })
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Claims or creates a pipeline stage for `name` (EVENT_PIPELINE.md
    /// phase 7) - persists until `unregisterStage`, so a client can
    /// pre-configure the stage's policy before any `stage = "name"`
    /// pattern registers.
    async fn register_stage(&self, ctx: &Context<'_>, name: StageName) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::RegisterStage { name })
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Removes the pipeline stage for `name` - its suppression state
    /// dies with it; patterns of that scope keep matching.
    async fn unregister_stage(&self, ctx: &Context<'_>, name: StageName) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::UnregisterStage { name })
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Starts a training session that captures a new profile with the given
    /// name.
    async fn train_profile(&self, ctx: &Context<'_>, name: PatternName) -> Result<bool> {
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
        label: Option<RecordLabel>,
    ) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::StartRecording {
                base_path: PathBuf::from(base_path),
                label,
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
    async fn set_record_label(
        &self,
        ctx: &Context<'_>,
        label: Option<RecordLabel>,
    ) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::SetRecordLabel(label))
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Replaces the missing-pin completion configuration. Takes a full
    /// config object - read the current one via the `completion` query,
    /// modify, write back (mutation applies from the next frame on).
    async fn set_completion(&self, ctx: &Context<'_>, config: CompletionConfig) -> Result<bool> {
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

    /// Shows or hides the daemon-embedded GUI window (`crates/gtk`,
    /// `--gui` mode) - the command is ignored on headless daemons.
    async fn set_gui_visible(&self, ctx: &Context<'_>, visible: bool) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::SetGuiVisible(visible))
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Quarter-turn the GUI canvas renders (0/90/180/270 degrees) -
    /// needed when the desktop shows the touchscreen rotated. Ignored
    /// on headless daemons.
    async fn set_display_rotation(
        &self,
        ctx: &Context<'_>,
        rotation: DisplayRotation,
    ) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::SetDisplayRotation(rotation))
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Registers a figure profile at runtime (DYNAMIC_PROFILES.md). The
    /// result union carries the assigned pattern ID or the typed
    /// rejection (`FINGERPRINT_COLLISION`, `PATTERN_NOT_FOUND`).
    async fn register_pattern(
        &self,
        ctx: &Context<'_>,
        input: RegisterPatternInput,
        owner: Option<Owner>,
        lifetime: Option<Lifetime>,
    ) -> Result<RegisterPatternResult> {
        let profile = ProfileFile::try_from(&input)?;
        let (command, rx) = ControlCommand::register(
            Registration::Pattern(profile),
            owner,
            lifetime.unwrap_or_default(),
        );
        ctx.data::<EngineHandle>()?
            .send_command(command)
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        match rx.await {
            Ok(Ok(RegistrationApplied::Profile(pattern_id))) => {
                Ok(RegisterPatternResult::Registered(PatternRegistered {
                    pattern_id,
                }))
            }
            Ok(Ok(_)) => Err(Error::new(
                "device loop returned a mismatched registration result",
            )),
            Ok(Err(error)) => Ok(RegisterPatternResult::Rejected(error.into())),
            Err(_) => Err(Error::new("device loop closed the registration reply")),
        }
    }

    /// Replaces a registered pattern's geometry/name while keeping its
    /// ID. Applies to new matches only.
    async fn update_profile(
        &self,
        ctx: &Context<'_>,
        id: PatternId,
        input: RegisterPatternInput,
    ) -> Result<bool> {
        let profile = ProfileFile::try_from(&input)?;
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
    async fn unregister_owner(&self, ctx: &Context<'_>, owner: Owner) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::UnregisterOwner { owner })
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }

    /// Registers an occupancy field set at runtime. The result union
    /// carries the typed rejection - `FIELDS_OVERLAP` exposes the
    /// offending field pair and intersection region (DYNAMIC_PROFILES.md).
    async fn register_field_set(
        &self,
        ctx: &Context<'_>,
        input: RegisterFieldSetInput,
        owner: Option<Owner>,
        lifetime: Option<Lifetime>,
    ) -> Result<RegisterFieldSetResult> {
        let set = FieldSet::from(&input);
        let name = set.name.clone();
        let (command, rx) = ControlCommand::register(
            Registration::FieldSet(set),
            owner,
            lifetime.unwrap_or_default(),
        );
        ctx.data::<EngineHandle>()?
            .send_command(command)
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        match rx.await {
            Ok(Ok(RegistrationApplied::FieldSet)) => {
                Ok(RegisterFieldSetResult::Registered(FieldSetRegistered {
                    name,
                }))
            }
            Ok(Ok(_)) => Err(Error::new(
                "device loop returned a mismatched registration result",
            )),
            Ok(Err(error)) => Ok(RegisterFieldSetResult::Rejected(error.into())),
            Err(_) => Err(Error::new("device loop closed the registration reply")),
        }
    }

    /// Removes a field set by name.
    async fn unregister_field_set(&self, ctx: &Context<'_>, name: FieldSetName) -> Result<bool> {
        ctx.data::<EngineHandle>()?
            .send_command(ControlCommand::UnregisterFieldSet { name })
            .await
            .map_err(|_| Error::new("control channel closed: device loop is not running"))?;
        Ok(true)
    }
}
