//! Control-command handling shared by the live modes: the drain loop,
//! registration commands, interface spawning and recording status.

use tangible_objects_engine::Engine;
use tangible_objects_engine::RecordingSession;
use tangible_objects_model::Lifetime;
use tangible_objects_model::PinCount;
use tangible_objects_model::Registration;
use tangible_objects_runtime::ControlCommand;
use tangible_objects_runtime::EngineHandle;
use tangible_objects_runtime::EngineRuntime;
use tangible_objects_runtime::LifecycleEvent;
use tangible_objects_runtime::ProfileInfo;
use tangible_objects_runtime::RecordingStatus;
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
            ControlCommand::TrainProfile(name) => {
                warn!(name, "remote training command - not yet wired");
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
                            println!("\nAufzeichnung gestartet: {}", started.base_path.display());
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
                    println!("\nAufzeichnung gestoppt: {recorded} Frames");
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
                println!(
                    "\nCompletion: mode={} radius={} max-missing={} persist={}",
                    config.mode, config.radius_factor, config.max_missing, config.persist
                );
            }
            ControlCommand::ResetMetrics => {
                runtime.reset_metrics();
            }
            ControlCommand::ShowMessage(text) => {
                println!("\n{}", text);
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
        } => {
            match *registration {
                Registration::Pattern(ref profile) => {
                    match engine.add_pattern(profile, owner.clone()) {
                        Ok(id) => {
                            println!(
                                "\nProfil registriert: {} (id {}, owner {:?}, {})",
                                profile.name, id, owner, lifetime
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
                                        println!(
                                            "\nWarnung: {} kollidiert mit {} \
                                             (k={}, min-dev {:.2}mm, {:?})",
                                            pair.a.as_str(),
                                            pair.b.as_str(),
                                            pair.contact_count,
                                            pair.min_max_deviation.as_f64(),
                                            pair.risk
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
                        }
                        Err(reason) => {
                            println!("\nProfil-Registrierung abgelehnt: {reason}");
                            return;
                        }
                    }
                }
                Registration::FieldSet(ref set) => {
                    let name = set.name.clone();
                    match engine.add_field_set(set.clone(), owner.clone()) {
                        Ok(()) => {
                            println!(
                                "\nFeld-Set registriert: {name} (owner {owner:?}, {lifetime})"
                            );
                            runtime.set_field_sets(engine.field_sets().to_vec());
                            runtime.emit_lifecycle(LifecycleEvent::FieldSetRegistered {
                                name: name.clone(),
                                owner: owner.clone(),
                            });
                        }
                        Err(reason) => {
                            println!("\nFeld-Set abgelehnt: {reason}");
                            return;
                        }
                    }
                }
            }
            if lifetime == Lifetime::Persistent {
                persist_registration(&registration);
            }
        }
        ControlCommand::UpdateProfile { id, profile } => {
            match engine.update_pattern(id, &profile) {
                Ok(()) => {
                    println!("\nProfil aktualisiert: {id} -> {}", profile.name);
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
                Err(reason) => println!("\nProfil-Update abgelehnt: {reason}"),
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
                println!("\nProfil entfernt: {id}");
            } else {
                println!("\nProfil {id} nicht gefunden");
            }
        }
        ControlCommand::UnregisterFieldSet { name } => {
            if engine.remove_field_set(&name) {
                runtime.set_field_sets(engine.field_sets().to_vec());
                unpersist("fields.d", &name);
                runtime.emit_lifecycle(LifecycleEvent::FieldSetUnregistered {
                    name: name.clone().into(),
                });
                println!("\nFeld-Set entfernt: {name}");
            } else {
                println!("\nFeld-Set {name} nicht gefunden");
            }
        }
        ControlCommand::UnregisterOwner { owner } => {
            let (patterns, field_sets) = engine.remove_by_owner(&owner);
            for (id, name) in &patterns {
                runtime.remove_profile(*id);
                unpersist("profiles.d", name);
                runtime.emit_lifecycle(LifecycleEvent::ProfileUnregistered { pattern_id: *id });
            }
            for name in &field_sets {
                unpersist("fields.d", name);
                runtime.emit_lifecycle(LifecycleEvent::FieldSetUnregistered {
                    name: name.clone().into(),
                });
            }
            runtime.set_field_sets(engine.field_sets().to_vec());
            println!(
                "\nOwner {owner} bereinigt: {} Profile, {} Feld-Sets entfernt",
                patterns.len(),
                field_sets.len()
            );
        }
        ControlCommand::ResetMetrics => runtime.reset_metrics(),
        ControlCommand::ShowMessage(text) => {
            println!("\n{}", text);
            runtime.set_message((!text.is_empty()).then_some(text));
        }
        other => warn!(?other, "command ignored"),
    }
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
        println!("GraphQL: http://{addr}/ (Endpoint: /graphql)");
    }
    if interfaces.dbus {
        tangible_objects_dbus::spawn(handle.clone());
        println!(
            "D-Bus: {} ({})",
            tangible_objects_dbus::BUS_NAME,
            tangible_objects_dbus::OBJECT_PATH
        );
    }
    if let Some(addr) = interfaces.mcp_addr {
        tangible_objects_mcp::spawn_http(handle.clone(), addr);
        println!("MCP: http://{addr}/mcp");
    }
}
