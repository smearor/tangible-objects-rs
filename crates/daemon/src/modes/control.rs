//! Control-command handling shared by the live modes: the drain loop,
//! registration commands, interface spawning and recording status.

use tangible_objects_engine::Engine;
use tangible_objects_engine::RecordingSession;
use tangible_objects_model::Lifetime;
use tangible_objects_model::PinCount;
#[cfg(feature = "gui")]
use tangible_objects_model::ProfileFile;
use tangible_objects_model::Registration;
use tangible_objects_model::RegistrationApplied;
use tangible_objects_runtime::ControlCommand;
use tangible_objects_runtime::EngineHandle;
use tangible_objects_runtime::EngineRuntime;
use tangible_objects_runtime::LifecycleEvent;
use tangible_objects_runtime::ProfileInfo;
use tangible_objects_runtime::RecordingStatus;
use tracing::info;
use tracing::warn;

use super::interfaces::Interfaces;
use super::store::persist_registration;
use super::store::unpersist;

/// Spawns the selected interface endpoints (GraphQL/D-Bus/MCP, Phase 2).
/// Drains pending control commands for [`run_live`].
///
/// Called both on device frames and on poll-timeouts so commands are
/// processed while the screen is idle - a queued `StopRecording` flushes the
/// writers immediately instead of waiting for the next touch.
pub(crate) fn drain_control_commands(
    runtime: &mut EngineRuntime,
    recording: &mut Option<RecordingSession>,
    engine: &mut Engine,
    device_name: Option<&str>,
) {
    while let Ok(command) = runtime.try_recv_command() {
        match command {
            ControlCommand::SetFilterMode { mode, .. } => {
                warn!(%mode, "filter mode command - only honored in --filter mode");
            }
            ControlCommand::RegisterStage { .. } | ControlCommand::UnregisterStage { .. } => {
                warn!("stage commands - only honored in --filter mode");
            }
            ControlCommand::TrainProfile(name) => {
                runtime.start_training(&name);
                runtime.set_message(Some(format!(
                    "Training '{name}': place the figure and hold still"
                )));
            }
            ControlCommand::StartRecording { base_path, label } => {
                if recording.is_some() {
                    warn!("recording already active - StartRecording ignored");
                } else {
                    match RecordingSession::start(
                        base_path.clone(),
                        device_name,
                        engine.calibration().as_ref(),
                        label,
                    ) {
                        Ok(started) => {
                            info!(path = %started.base_path.display(), "recording started");
                            *recording = Some(started);
                            runtime.emit_lifecycle(LifecycleEvent::RecordingStarted);
                        }
                        Err(e) => warn!(error = %e, "recording could not be started"),
                    }
                }
            }
            ControlCommand::StopRecording => match recording.take() {
                Some(stopped) => {
                    let recorded = stopped.recorded_frames();
                    if let Err(e) = stopped.finish() {
                        warn!(error = %e, "recording flush failed");
                    }
                    runtime.emit_lifecycle(LifecycleEvent::RecordingStopped { frames: recorded });
                    info!(%recorded, "recording stopped");
                }
                None => warn!("no recording active - StopRecording ignored"),
            },
            ControlCommand::SetRecordLabel(new_label) => match recording {
                Some(session) => session.set_label(new_label),
                None => warn!("no recording active - SetRecordLabel ignored"),
            },
            ControlCommand::SetCompletion(config) => {
                engine.set_completion(config);
                runtime.set_completion(config);
                runtime.emit_lifecycle(LifecycleEvent::CompletionChanged(config));
                info!(
                    "completion: mode={} radius={} max-missing={} persist={}",
                    config.mode, config.radius_factor, config.max_missing, config.persist
                );
            }
            ControlCommand::ResetMetrics => {
                runtime.reset_metrics();
            }
            ControlCommand::ShowMessage(text) => {
                info!(%text, "message");
                runtime.set_message((!text.is_empty()).then_some(text));
            }
            other => apply_registration(runtime, engine, other),
        }
    }
}

/// Applies `Register`/`Unregister*`/`UpdateProfile` control commands to
/// engine and runtime; unrelated commands are logged and ignored.
pub(crate) fn apply_registration(
    runtime: &mut EngineRuntime,
    engine: &mut Engine,
    command: ControlCommand,
) {
    match command {
        ControlCommand::Register {
            registration,
            owner,
            lifetime,
            reply,
        } => {
            let result = match *registration {
                Registration::Pattern(ref profile) => {
                    match engine.add_pattern(profile, owner.clone()) {
                        Ok(id) => {
                            info!(
                                name = %profile.name, %id, ?owner, %lifetime,
                                "profile registered"
                            );
                            // Warn when the new profile's fingerprint is
                            // geometrically close to an existing one - the
                            // phantom surface for partial spawns.
                            if let Some(new_pattern) = engine.patterns().iter().find(|p| p.id == id)
                            {
                                for existing in engine.patterns() {
                                    if existing.id == id {
                                        continue;
                                    }
                                    if let Some(pair) =
                                        tangible_objects_analysis::SimilarityPair::closest(
                                            new_pattern,
                                            existing,
                                        )
                                        && pair.risk != tangible_objects_analysis::MatchRisk::Safe
                                    {
                                        warn!(
                                            a = %pair.a, b = %pair.b,
                                            k = %pair.contact_count,
                                            min_dev_mm = %pair.min_max_deviation.as_f64(),
                                            risk = ?pair.risk,
                                            "profile fingerprint collision risk"
                                        );
                                    }
                                }
                            }
                            runtime.add_profile(ProfileInfo {
                                pattern_id: id,
                                name: profile.name.clone(),
                                num_points: PinCount::new(profile.num_points),
                                owner: owner.clone(),
                                spawn_on_partial: profile.spawn_on_partial,
                                spawn_stability: profile.spawn_stability,
                                stage: profile.stage.clone(),
                            });
                            runtime.emit_lifecycle(LifecycleEvent::ProfileRegistered {
                                pattern_id: id,
                                name: profile.name.clone(),
                                owner,
                            });
                            Ok(RegistrationApplied::Profile(id))
                        }
                        Err(reason) => {
                            warn!(%reason, "profile registration rejected");
                            Err(reason)
                        }
                    }
                }
                Registration::FieldSet(ref set) => {
                    let name = set.name.clone();
                    match engine.add_field_set(set.clone(), owner.clone()) {
                        Ok(()) => {
                            info!(%name, ?owner, %lifetime, "field set registered");
                            runtime.set_field_sets(engine.field_sets().to_vec());
                            runtime.emit_lifecycle(LifecycleEvent::FieldSetRegistered {
                                name: name.clone(),
                                owner: owner.clone(),
                            });
                            Ok(RegistrationApplied::FieldSet)
                        }
                        Err(reason) => {
                            warn!(%reason, "field set rejected");
                            Err(reason)
                        }
                    }
                }
            };
            if result.is_ok() && lifetime == Lifetime::Persistent {
                persist_registration(&registration);
            }
            let _ = reply.send(result);
        }
        ControlCommand::UpdateProfile { id, profile } => {
            match engine.update_pattern(id, &profile) {
                Ok(()) => {
                    info!(%id, name = %profile.name, "profile updated");
                    let owner = engine.pattern_owner(id).cloned();
                    runtime.remove_profile(id);
                    runtime.add_profile(ProfileInfo {
                        pattern_id: id,
                        name: profile.name.clone(),
                        num_points: PinCount::new(profile.num_points),
                        spawn_on_partial: profile.spawn_on_partial,
                        spawn_stability: profile.spawn_stability,
                        owner: owner.clone(),
                        stage: profile.stage.clone(),
                    });
                    runtime.emit_lifecycle(LifecycleEvent::ProfileRegistered {
                        pattern_id: id,
                        name: profile.name.clone(),
                        owner,
                    });
                }
                Err(reason) => warn!(%reason, "profile update rejected"),
            }
        }
        ControlCommand::UnregisterPattern { id } => {
            let name = engine
                .patterns()
                .iter()
                .find(|pattern| pattern.id == id)
                .map(|pattern| pattern.name.clone());
            if engine.remove_pattern(id) {
                runtime.remove_profile(id);
                if let Some(name) = name {
                    unpersist("profiles.d", name.as_str());
                }
                runtime.emit_lifecycle(LifecycleEvent::ProfileUnregistered { pattern_id: id });
                info!(%id, "profile removed");
            } else {
                warn!(%id, "profile not found");
            }
        }
        ControlCommand::UnregisterFieldSet { name } => {
            if engine.remove_field_set(&name) {
                runtime.set_field_sets(engine.field_sets().to_vec());
                unpersist("fields.d", name.as_str());
                runtime.emit_lifecycle(LifecycleEvent::FieldSetUnregistered { name: name.clone() });
                info!(%name, "field set removed");
            } else {
                warn!(%name, "field set not found");
            }
        }
        ControlCommand::UnregisterOwner { owner } => {
            let (patterns, field_sets) = engine.remove_by_owner(&owner);
            for (id, name) in &patterns {
                runtime.remove_profile(*id);
                unpersist("profiles.d", name.as_str());
                runtime.emit_lifecycle(LifecycleEvent::ProfileUnregistered { pattern_id: *id });
            }
            for name in &field_sets {
                unpersist("fields.d", name.as_str());
                runtime.emit_lifecycle(LifecycleEvent::FieldSetUnregistered { name: name.clone() });
            }
            runtime.set_field_sets(engine.field_sets().to_vec());
            info!(
                %owner, profiles = patterns.len(), field_sets = field_sets.len(),
                "owner cleaned up"
            );
        }
        ControlCommand::ResetMetrics => runtime.reset_metrics(),
        ControlCommand::ShowMessage(text) => {
            info!(%text, "message");
            runtime.set_message((!text.is_empty()).then_some(text));
        }
        #[cfg(feature = "gui")]
        ControlCommand::SetGuiVisible(visible) => runtime.set_gui_visible(visible),
        other => warn!(?other, "command ignored"),
    }
}

/// Advances remote profile training: ticks the runtime's
/// [`TrainingState`][tangible_objects_runtime::TrainingState] with the
/// frame's touches and applies a persistent `Register` once the point
/// set stabilized (capture → `add_pattern` + lifecycle + persist).
pub(crate) fn tick_training(
    runtime: &mut EngineRuntime,
    engine: &mut Engine,
    touches: &[tangible_objects_model::TouchPoint],
) {
    if let Some(profile) = runtime.training_tick(touches) {
        info!(name = %profile.name, "training completed");
        runtime.set_message(None);
        let (command, _rx) =
            ControlCommand::register(Registration::Pattern(profile), None, Lifetime::Persistent);
        apply_registration(runtime, engine, command);
    }
}

/// `GuiHooks::register_profile` implementation - registers a
/// completed remote-training capture as a persistent pattern.
/// Only the GUI host calls it (`gui` feature).
#[cfg(feature = "gui")]
pub(crate) fn register_profile_hook(
    runtime: &mut EngineRuntime,
    engine: &mut Engine,
    profile: ProfileFile,
) {
    let (command, _rx) =
        ControlCommand::register(Registration::Pattern(profile), None, Lifetime::Persistent);
    apply_registration(runtime, engine, command);
}

/// Live snapshot of a recording session for the interface-facing status.
pub(crate) fn status_of_session(session: &RecordingSession) -> RecordingStatus {
    RecordingStatus {
        active: true,
        label: session.label().cloned(),
        recorded_frames: session.recorded_frames(),
        dropped_frames: session.dropped_frames(),
        dropped_events: session.dropped_events(),
    }
}

pub(crate) fn spawn_interfaces(handle: &EngineHandle, interfaces: &Interfaces) {
    if let Some(addr) = interfaces.graphql_addr {
        tangible_objects_graphql::spawn(handle.clone(), addr);
        info!(%addr, "GraphQL endpoint listening (http://{addr}/graphql)");
    }
    if interfaces.dbus {
        tangible_objects_dbus::spawn(handle.clone());
        info!(
            bus = tangible_objects_dbus::BUS_NAME,
            path = tangible_objects_dbus::OBJECT_PATH,
            "D-Bus interface listening"
        );
    }
    if let Some(addr) = interfaces.mcp_addr {
        tangible_objects_mcp::spawn_http(handle.clone(), addr);
        info!(%addr, "MCP endpoint listening (http://{addr}/mcp)");
    }
}
