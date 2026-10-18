//! Daemon modes: replay, live detection, training and device listing.

use std::collections::HashSet;
use std::io::Write;
use std::net::SocketAddr;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use miette::IntoDiagnostic;
use miette::Result;
use tangible_objects_engine::Engine;
use tangible_objects_engine::EngineEvent;
use tangible_objects_engine::EvtestReplay;
use tangible_objects_engine::FilterProxy;
use tangible_objects_engine::LegacyMode;
use tangible_objects_engine::RecordingSession;
use tangible_objects_engine::TouchDevice;
use tangible_objects_engine::VirtualTouchscreen;
use tangible_objects_model::Calibration;
use tangible_objects_model::CompletionConfig;
use tangible_objects_model::CompletionMode;
use tangible_objects_model::DeviceRegistry;
use tangible_objects_model::FieldSet;
use tangible_objects_model::FrameCount;
use tangible_objects_model::FrameNumber;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Milliseconds;
use tangible_objects_model::Owner;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::RecordLabel;
use tangible_objects_model::Registration;
use tangible_objects_model::TangiblePattern;
use tangible_objects_model::TouchPointSlot;
use tangible_objects_model::Vec2;
use tangible_objects_runtime::ControlCommand;
use tangible_objects_runtime::DeviceInfo;
use tangible_objects_runtime::EngineHandle;
use tangible_objects_runtime::EngineRuntime;
use tangible_objects_runtime::ProfileInfo;
use tangible_objects_runtime::RecordingStatus;
use tracing::warn;

/// Effective calibration for a live device.
///
/// ABS_INFO (`device.calibration()`) is authoritative for axis bounds, but
/// when the device's profile declares a resolution override the measured
/// physical resolution wins - some panels misreport their units/mm in
/// firmware (e.g. the SiS panel reporting 7/12 while driving a 65" display).
/// Returns the reported calibration when the device has no profile or the
/// profile has no override.
pub fn effective_calibration(
    device: &TouchDevice,
    device_profiles: Option<&Path>,
) -> Option<Calibration> {
    let reported = device.calibration()?;
    let mut registry = DeviceRegistry::builtin();
    if let Some(dir) = device_profiles {
        registry = registry.with_dir(dir);
    }
    let id = device.inner().input_id();
    let profile = registry.find_by_ids(id.vendor(), id.product())?;
    let (x_res, y_res) = profile.resolution_override()?;
    println!(
        "Resolution-Override aus Geräteprofil: X={:.3}, Y={:.3} units/mm",
        x_res.as_f64(),
        y_res.as_f64()
    );
    Some(Calibration::with_bounds(
        x_res,
        y_res,
        reported.x_min,
        reported.x_max,
        reported.y_min,
        reported.y_max,
    ))
}

/// Loads figure patterns from a profile directory (default: `profiles/`).
pub fn load_patterns(profiles_dir: Option<&str>) -> Vec<TangiblePattern> {
    let dir = profiles_dir.unwrap_or("profiles");
    let mut dir_result = ProfileFile::load_dir(Path::new(dir));
    // `Lifetime::Persistent` registrations live in the runtime store and
    // reload with the static set.
    if let Some(store) = runtime_store_dir("profiles.d")
        && store.is_dir()
    {
        let mut stored = ProfileFile::load_dir(&store);
        dir_result.profiles.append(&mut stored.profiles);
        dir_result.errors.append(&mut stored.errors);
    }
    for (_, profile) in &dir_result.profiles {
        if profile.enabled {
            println!(
                "Profil geladen: {} ({} Punkte, {} Abstände)",
                profile.name,
                profile.num_points,
                profile.sorted_sides.len()
            );
        } else {
            println!("Profil deaktiviert: {}", profile.name);
        }
    }
    for (path, error) in &dir_result.errors {
        eprintln!(
            "Warnung: Profil {} konnte nicht geladen werden: {}",
            path.display(),
            error
        );
    }
    let patterns = dir_result.into_patterns();
    if patterns.is_empty() {
        eprintln!(
            "Warnung: Keine Profile in {} gefunden. Erstelle ein Profil mit --train.",
            dir
        );
    }
    patterns
}

/// Replays an `evtest` recording through the detection pipeline.
///
/// With `record_base` a [`RecordingSession`] additionally writes
/// `<base>.jsonl` + `<base>.evtest`; event timestamps come from the input
/// file itself (offset to the first event), so the artifacts are
/// deterministic.
pub fn run_replay(
    path: &Path,
    patterns: &[TangiblePattern],
    record_base: Option<&Path>,
    label: Option<RecordLabel>,
) -> Result<()> {
    let mut replay = EvtestReplay::open(path).into_diagnostic()?;
    let mut engine = Engine::new(patterns.to_vec());

    if let Some(cal) = replay.calibration() {
        println!(
            "Kalibrierung: X={:.1} units/mm, Y={:.1} units/mm",
            cal.x_res, cal.y_res
        );
        engine.set_calibration(cal);
    } else {
        eprintln!(
            "Warnung: Keine Kalibrierung im evtest-Header gefunden. Profile müssen in Rohkoordinaten vorliegen."
        );
    }

    let mut session = match record_base {
        Some(base) => Some(
            RecordingSession::start(
                base.to_path_buf(),
                replay.device_name(),
                engine.calibration().as_ref(),
                label,
            )
            .into_diagnostic()?,
        ),
        None => None,
    };

    let mut frame_no: u64 = 0;
    let mut detected_frames: u64 = 0;
    let mut first_timestamp: Option<f64> = None;

    while let Some((event, timestamp)) = replay.next_event_timed() {
        let offset = timestamp - *first_timestamp.get_or_insert(timestamp);
        if let Some(session) = &session {
            session.record_event(&event, offset);
        }
        let engine_frame = if session.is_some() {
            engine.process_with_diagnostics(&event)
        } else {
            engine.process(&event)
        };
        let Some(frame) = engine_frame else {
            continue;
        };
        let added = frame
            .events
            .iter()
            .any(|e| matches!(e, EngineEvent::TangibleAdded(_)));
        for engine_event in &frame.events {
            match engine_event {
                EngineEvent::TangibleAdded(_) => println!(
                    "[Frame {frame_no}] Figur erkannt ({} aktiv)",
                    engine.tracked().len()
                ),
                EngineEvent::TangibleRemoved(_) if engine.tracked().is_empty() => {
                    println!("[Frame {frame_no}] Figur verloren");
                }
                EngineEvent::FieldPlaced { set, field } => {
                    println!("[Frame {frame_no}] Feld belegt: {set}.{field}")
                }
                EngineEvent::FieldLifted { set, field } => {
                    println!("[Frame {frame_no}] Feld frei: {set}.{field}")
                }
                EngineEvent::FieldCountChanged { set, field, count } => {
                    println!("[Frame {frame_no}] Feld {set}.{field}: {count} Kontakt(e)")
                }
                _ => {}
            }
        }

        let active = engine.tracked();
        if !active.is_empty() {
            detected_frames += 1;
            if detected_frames.is_multiple_of(50) || added {
                for t in active {
                    let deg = t.rotation_rad.to_degrees();
                    println!(
                        "  {} (id={}): center=({:.0}, {:.0}) rotation={:.1}°",
                        t.pattern_name, t.pattern_id, t.center.x, t.center.y, deg
                    );
                }
            }
        }
        if let Some(session) = &mut session {
            session.record_frame(
                FrameNumber::new(frame_no),
                Milliseconds::from(Duration::from_secs_f64(offset)),
                &frame,
                engine.tracked(),
            );
        }
        frame_no += 1;
    }

    println!("\n--- Zusammenfassung ---");
    println!("Frames gesamt: {frame_no}");
    println!("Frames mit Erkennung: {detected_frames}");
    if let Some(session) = session {
        let recorded = session.recorded_frames();
        let dropped_frames = session.dropped_frames();
        let dropped_events = session.dropped_events();
        session.finish().into_diagnostic()?;
        println!(
            "Aufzeichnung: {recorded} Frames (gedroppt: {dropped_frames} Frames, {dropped_events} Events)"
        );
    }
    Ok(())
}

/// Optional interface endpoints for the live mode.
#[derive(Debug, Default, Clone)]
pub struct Interfaces {
    /// GraphQL listen address, if the HTTP/WebSocket API is enabled.
    pub graphql_addr: Option<SocketAddr>,
    /// Whether to serve the D-Bus session-bus interface.
    pub dbus: bool,
    /// MCP streamable-HTTP listen address, if enabled.
    pub mcp_addr: Option<SocketAddr>,
}

/// Optional detection settings bundled for the run_* modes.
#[derive(Debug, Default, Clone)]
pub struct DetectionConfig {
    /// Edge ghost margin in millimeters (`--edge-filter-mm`).
    pub edge: Option<Millimeters>,
    /// Occupancy field sets (`--fields`).
    pub fields: Vec<FieldSet>,
    /// Missing-pin completion config (`--completion-*` flags).
    pub completion: CompletionConfig,
}

/// Default path of the completion config file:
/// `$XDG_CONFIG_HOME/tangible-objects/completion.toml`, falling back to
/// `~/.config/tangible-objects/completion.toml`.
pub fn completion_config_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("tangible-objects").join("completion.toml"))
}

/// Loads the completion config from [`completion_config_path`]; falls
/// back to [`CompletionConfig::default`] when the file is absent or
/// unreadable. CLI `--completion-*` flags override individual fields on
/// top of this.
pub fn load_completion() -> CompletionConfig {
    let Some(path) = completion_config_path() else {
        return CompletionConfig::default();
    };
    match std::fs::read_to_string(&path) {
        Ok(contents) => match toml::from_str::<CompletionConfig>(&contents) {
            Ok(config) => {
                println!(
                    "Completion-Config geladen: {} (mode={} radius={} max-missing={} persist={})",
                    path.display(),
                    config.mode,
                    config.radius_factor,
                    config.max_missing,
                    config.persist
                );
                config
            }
            Err(e) => {
                warn!(path = %path.display(), error = %e, "completion config unreadable - defaults used");
                CompletionConfig::default()
            }
        },
        Err(_) => CompletionConfig::default(),
    }
}

/// Loads occupancy field sets from a file or directory (`--fields`).
///
/// A `.toml` file loads one set, a directory loads all sets inside.
/// Invalid files are reported and skipped.
pub fn load_field_sets(path: Option<&str>) -> Vec<FieldSet> {
    let mut sets = Vec::new();
    // Persisted field sets (`Lifetime::Persistent`) reload with the
    // static `--fields` set.
    if let Some(store) = runtime_store_dir("fields.d")
        && store.is_dir()
    {
        let (stored, errors) = FieldSet::load_dir(&store);
        for (file, error) in &errors {
            eprintln!(
                "Warnung: Feld-Set {} konnte nicht geladen werden: {error}",
                file.display()
            );
        }
        sets.extend(stored);
    }
    let Some(path) = path else {
        return sets;
    };
    let path = Path::new(path);
    let file_sets = if path.is_dir() {
        let (file_sets, errors) = FieldSet::load_dir(path);
        for (file, error) in &errors {
            eprintln!(
                "Warnung: Feld-Set {} konnte nicht geladen werden: {error}",
                file.display()
            );
        }
        file_sets
    } else {
        match FieldSet::load(path) {
            Ok(set) => vec![set],
            Err(error) => {
                eprintln!(
                    "Warnung: Feld-Set {} konnte nicht geladen werden: {error}",
                    path.display()
                );
                Vec::new()
            }
        }
    };
    sets.extend(file_sets);
    for set in &sets {
        println!(
            "Feld-Set geladen: {} ({} Felder)",
            set.name,
            set.fields.len()
        );
    }
    sets
}

/// Live detection loop on a real evdev device.
///
/// `interfaces` selects which Phase-2 interfaces are served on dedicated
/// threads (GraphQL HTTP/WebSocket, D-Bus session bus).
pub fn run_live(
    path: &Path,
    patterns: &[TangiblePattern],
    interfaces: Interfaces,
    config: DetectionConfig,
    device_profiles: Option<&Path>,
) -> Result<()> {
    let mut device = TouchDevice::open(path).into_diagnostic()?;

    println!("Live-Modus: {}", path.display());
    if let Some(name) = device.name() {
        println!("Gerät: {name}");
    }

    let mut engine = Engine::new(patterns.to_vec());
    if let Some(cal) = effective_calibration(&device, device_profiles) {
        println!(
            "Kalibrierung: X={:.1} units/mm, Y={:.1} units/mm",
            cal.x_res, cal.y_res
        );
        engine.set_calibration(cal);
    } else {
        eprintln!(
            "Warnung: Keine Kalibrierung vom Gerät lesbar. Profile müssen in Rohkoordinaten vorliegen."
        );
    }
    if let Some(mm) = config.edge {
        engine.set_edge_margin(mm);
        println!("Edge-Filter: {mm} mm");
    }
    engine.set_fields(config.fields);
    engine.set_completion(config.completion);
    println!("Ctrl+C zum Beenden\n");

    // Event hub for interface crates (GraphQL/D-Bus/MCP, Phase 2).
    // No set definitions are loaded yet - the set tracker idles.
    let profiles: Vec<ProfileInfo> = patterns.iter().map(ProfileInfo::from).collect();
    let (mut runtime, handle) = EngineRuntime::new(Vec::new(), profiles);
    runtime.set_completion(config.completion);
    runtime.set_field_sets(engine.field_sets().to_vec());
    spawn_interfaces(&handle, &interfaces);
    runtime.set_device_info(DeviceInfo::of(device.name(), device.calibration().as_ref()));

    let mut frame: u64 = 0;
    let session_start = Instant::now();
    let mut recording: Option<RecordingSession> = None;
    let device_fd = device.as_raw_fd();

    loop {
        // Wake periodically even without device events: queued control
        // commands (start/stop recording) must not wait for input - a
        // StopRecording on an idle screen has to flush immediately.
        let mut pfd = libc::pollfd {
            fd: device_fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = unsafe { libc::poll(&mut pfd, 1, 250) };
        if ready < 0 {
            if unsafe { *libc::__errno_location() } == libc::EINTR {
                continue;
            }
            eprintln!("poll Fehler");
            break;
        }
        if ready == 0 {
            drain_control_commands(&mut runtime, &mut recording, &mut engine, device.name());
            runtime.set_recording_status(recording.as_ref().map(status_of_session));
            runtime.publish_snapshot(engine.touches(), engine.tracked());
            continue;
        }

        let events = match device.fetch_events() {
            Ok(events) => events,
            Err(e) => {
                eprintln!("Lesefehler: {e}");
                break;
            }
        };

        for raw in events {
            if let Some(session) = &recording {
                session.record_event(&raw, session_start.elapsed().as_secs_f64());
            }
            let engine_frame = if recording.is_some() {
                engine.process_with_diagnostics(&raw)
            } else {
                engine.process(&raw)
            };
            if let Some(engine_frame) = engine_frame {
                frame += 1;
                runtime.publish(&engine_frame, engine.tracked());
                drain_control_commands(&mut runtime, &mut recording, &mut engine, device.name());

                if let Some(session) = &mut recording {
                    session.record_frame(
                        FrameNumber::new(frame),
                        Milliseconds::from(session_start.elapsed()),
                        &engine_frame,
                        engine.tracked(),
                    );
                }
                runtime.set_recording_status(recording.as_ref().map(status_of_session));

                let active = engine.tracked();
                print!("\x1b[2J\x1b[H");
                println!(
                    "Frame {frame} | {} Touch-Punkt(e) | {} Figur(en) | Rec: {}",
                    engine.touch_count(),
                    active.len(),
                    recording
                        .as_ref()
                        .and_then(|s| s.label().map(|l| l.as_str().to_string()))
                        .unwrap_or_else(|| if recording.is_some() {
                            "an".to_string()
                        } else {
                            "aus".to_string()
                        }),
                );
                for t in active {
                    let deg = t.rotation_rad.to_degrees();
                    println!(
                        "  {} (id={}): center=({:.0}, {:.0}) rotation={:.1}°",
                        t.pattern_name, t.pattern_id, t.center.x, t.center.y, deg
                    );
                }
                std::io::stdout().flush().ok();
            }
        }
    }
    Ok(())
}

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

/// Runtime store for `Lifetime::Persistent` registrations:
/// `~/.config/tangible-objects/profiles.d/` resp. `fields.d/`.
fn runtime_store_dir(kind: &str) -> Option<PathBuf> {
    completion_config_path()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .map(|base| base.join(kind))
}

/// Persists a registered profile/field set into the runtime store.
fn persist_registration(registration: &Registration) {
    let Some(store) = runtime_store_dir(match registration {
        Registration::Pattern(_) => "profiles.d",
        Registration::FieldSet(_) => "fields.d",
    }) else {
        return;
    };
    let (name, result) = match registration {
        Registration::Pattern(profile) => (
            profile.name.as_str().to_string(),
            std::fs::create_dir_all(&store).and_then(|_| {
                profile
                    .save(&store.join(format!("{}.toml", profile.name)))
                    .map_err(std::io::Error::other)
            }),
        ),
        Registration::FieldSet(set) => (
            set.name.as_str().to_string(),
            std::fs::create_dir_all(&store).and_then(|_| {
                std::fs::write(
                    store.join(format!("{}.toml", set.name)),
                    toml::to_string_pretty(set).unwrap_or_default(),
                )
            }),
        ),
    };
    match result {
        Ok(()) => println!("\nGespeichert: {name}"),
        Err(e) => warn!(error = %e, name, "persist failed"),
    }
}

/// Deletes a persisted registration file, if it exists.
fn unpersist(store_kind: &str, name: &str) {
    if let Some(path) = runtime_store_dir(store_kind).map(|dir| dir.join(format!("{name}.toml")))
        && path.exists()
        && let Err(e) = std::fs::remove_file(&path)
    {
        warn!(error = %e, path = %path.display(), "unpersist failed");
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
                                num_points: profile.num_points as u32,
                                owner,
                                spawn_on_partial: profile.spawn_on_partial,
                                spawn_stability: profile.spawn_stability,
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
                    runtime.remove_profile(id);
                    runtime.add_profile(ProfileInfo {
                        pattern_id: id,
                        name: profile.name.clone(),
                        num_points: profile.num_points as u32,
                        spawn_on_partial: profile.spawn_on_partial,
                        spawn_stability: profile.spawn_stability,
                        owner: engine
                            .pattern_owner(id)
                            .map(Owner::to_string)
                            .map(Owner::new),
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
                println!("\nProfil entfernt: {id}");
            } else {
                println!("\nProfil {id} nicht gefunden");
            }
        }
        ControlCommand::UnregisterFieldSet { name } => {
            if engine.remove_field_set(&name) {
                runtime.set_field_sets(engine.field_sets().to_vec());
                unpersist("fields.d", &name);
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
            }
            for name in &field_sets {
                unpersist("fields.d", name);
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

/// Filter-proxy mode (FILTER_PROXY_VIRTUAL.md): grabs the real device and
/// re-emits its event stream on a uinput clone, minus the slots a matched
/// tangible occupies.
///
/// Only exact matches suppress - partial candidates carry a widened
/// tolerance that would eat finger touches resembling a profile. A matched
/// slot stays suppressed across transient match loss until its tracking
/// contact ends or the filter's hysteresis expires (Figur7/8-style flappy
/// pins must not leak to the OS). `SetFilterMode` control commands switch
/// the suppression policy live (off/immediate/gated/guarded).
pub fn run_filter(
    path: &Path,
    patterns: &[TangiblePattern],
    interfaces: Interfaces,
    stability: Option<FrameCount>,
    legacy: Option<LegacyMode>,
    config: DetectionConfig,
    device_profiles: Option<&Path>,
) -> Result<()> {
    let mut device = TouchDevice::open(path).into_diagnostic()?;
    println!("Filter-Proxy: {}", path.display());
    if let Some(name) = device.name() {
        println!("Gerät: {name}");
    }

    let mut engine = Engine::new(patterns.to_vec());
    if let Some(cal) = effective_calibration(&device, device_profiles) {
        println!(
            "Kalibrierung: X={:.1} units/mm, Y={:.1} units/mm",
            cal.x_res, cal.y_res
        );
        engine.set_calibration(cal);
    }
    if let Some(mm) = config.edge {
        engine.set_edge_margin(mm);
        println!("Edge-Filter: {mm} mm");
    }
    engine.set_fields(config.fields);
    engine.set_completion(config.completion);

    let mut virtual_device = VirtualTouchscreen::create(device.inner()).into_diagnostic()?;
    device.grab().into_diagnostic()?;
    println!("EVIOCGRAB aktiv - alle Events laufen über die Proxy");
    println!("uinput-Klon erstellt - Tangibles werden unterdrückt");
    println!("Ctrl+C zum Beenden\n");

    let profiles: Vec<ProfileInfo> = patterns.iter().map(ProfileInfo::from).collect();
    let (mut runtime, handle) = EngineRuntime::new(Vec::new(), profiles);
    runtime.set_completion(config.completion);
    runtime.set_field_sets(engine.field_sets().to_vec());
    spawn_interfaces(&handle, &interfaces);
    runtime.set_device_info(DeviceInfo::of(device.name(), device.calibration().as_ref()));

    let mut proxy = FilterProxy::new();
    if let Some(gate) = stability {
        proxy.set_mode(tangible_objects_model::FilterMode::default(), Some(gate));
        println!("Stabilitäts-Gate: {gate} Frames");
    }
    if let Some(legacy_mode) = legacy {
        proxy.set_legacy_mode(legacy_mode);
        println!("Legacy-Events (BTN_TOUCH/ABS_X/Y): {legacy_mode}");
    }
    let mut frame: u64 = 0;
    let device_fd = device.as_raw_fd();
    loop {
        // Commands run on every loop turn - including poll timeouts - so a
        // queued SetFilterMode applies while the device is idle.
        while let Ok(command) = runtime.try_recv_command() {
            match command {
                ControlCommand::SetFilterMode { mode, stability } => {
                    proxy.set_mode(mode, stability);
                    println!("\nFilter-Modus: {mode}");
                }
                ControlCommand::SetCompletion(config) => {
                    engine.set_completion(config);
                    runtime.set_completion(config);
                    println!(
                        "\nCompletion: mode={} radius={} max-missing={} persist={}",
                        config.mode, config.radius_factor, config.max_missing, config.persist
                    );
                }
                other => apply_registration(&mut runtime, &mut engine, other),
            }
        }

        let mut pfd = libc::pollfd {
            fd: device_fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = unsafe { libc::poll(&mut pfd, 1, 250) };
        if ready < 0 {
            if unsafe { *libc::__errno_location() } == libc::EINTR {
                continue;
            }
            eprintln!("poll Fehler");
            break;
        }
        if ready == 0 {
            continue;
        }

        let events = match device.fetch_events() {
            Ok(events) => events,
            Err(e) => {
                eprintln!("Lesefehler: {e}");
                break;
            }
        };
        for raw in events {
            let engine_frame = engine.process(&raw);
            proxy.push(raw);
            let Some(engine_frame) = engine_frame else {
                continue;
            };
            frame += 1;

            // Exact matches always suppress; under CompletionMode::Exact a
            // completed partial suppresses its matched *and* claimed slots.
            let completion_exact = engine.completion().mode == CompletionMode::Exact;
            let mut suppressed: HashSet<TouchPointSlot> = engine_frame
                .matches
                .iter()
                .flat_map(|matched| {
                    if !matched.is_partial()
                        || (completion_exact && !matched.claimed_indices.is_empty())
                    {
                        matched.all_indices().collect::<Vec<usize>>()
                    } else {
                        Vec::new()
                    }
                })
                .map(|index| engine_frame.touches[index].slot)
                .collect();
            // Contacts inside `suppress`-enabled field sets never reach
            // the OS either (SINGLE_CONTACT_FIELD.md).
            suppressed.extend(engine.field_suppressed_slots(&engine_frame.touches));

            let out = proxy.finish(&suppressed);
            virtual_device.emit(&out).into_diagnostic()?;

            print!(
                "\rFrame {frame} | {} Touch | {} Figur | {} unterdrückt     ",
                engine_frame.touches.len(),
                engine_frame.matches.len(),
                suppressed.len()
            );
            std::io::stdout().flush().ok();
        }
    }
    Ok(())
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

/// Number of frames a keypress label applies to (~1 s at 120 Hz report rate).
const LABEL_FRAME_COUNT: u64 = 120;

/// Recording mode: like [`run_live`], additionally writing every frame to
/// `<base>.jsonl` and every raw event to `<base>.evtest` (TRAINING.md).
///
/// Matching runs through [`Engine::process_with_diagnostics`], so every
/// tested candidate lands in the frame record. Enter stops the recording
/// cleanly (drains and flushes both writers); Ctrl+C also ends it but may
/// lose the buffered tail. `SetRecordLabel`/`StopRecording` control commands
/// are honored live - API-driven labeling for application sessions.
pub fn run_record(
    path: &Path,
    base: &Path,
    label: Option<RecordLabel>,
    patterns: &[TangiblePattern],
    interfaces: Interfaces,
    config: DetectionConfig,
    device_profiles: Option<&Path>,
) -> Result<()> {
    let mut device = TouchDevice::open(path).into_diagnostic()?;

    println!("Record-Modus: {}", path.display());
    if let Some(name) = device.name() {
        println!("Gerät: {name}");
    }

    let mut engine = Engine::new(patterns.to_vec());
    if let Some(cal) = effective_calibration(&device, device_profiles) {
        println!(
            "Kalibrierung: X={:.1} units/mm, Y={:.1} units/mm",
            cal.x_res, cal.y_res
        );
        engine.set_calibration(cal);
    } else {
        eprintln!(
            "Warnung: Keine Kalibrierung vom Gerät lesbar. Positionen werden als Rohwerte aufgezeichnet."
        );
    }
    if let Some(mm) = config.edge {
        engine.set_edge_margin(mm);
        println!("Edge-Filter: {mm} mm");
    }
    engine.set_fields(config.fields);
    engine.set_completion(config.completion);

    let mut session = RecordingSession::start(
        base.to_path_buf(),
        device.name(),
        engine.calibration().as_ref(),
        label,
    )
    .into_diagnostic()?;
    println!(
        "Aufzeichnung: {}.jsonl + {}.evtest",
        base.display(),
        base.display()
    );
    println!(
        "Enter ohne Text beendet - <label>+Enter labelt die nächsten {LABEL_FRAME_COUNT} Frames.\n"
    );

    let profiles: Vec<ProfileInfo> = patterns.iter().map(ProfileInfo::from).collect();
    let (mut runtime, handle) = EngineRuntime::new(Vec::new(), profiles);
    runtime.set_completion(config.completion);
    runtime.set_field_sets(engine.field_sets().to_vec());
    spawn_interfaces(&handle, &interfaces);
    runtime.set_device_info(DeviceInfo::of(device.name(), device.calibration().as_ref()));

    let mut frame_no: u64 = 0;
    let session_start = Instant::now();

    // Label input via poll() - wait on device fd and stdin simultaneously.
    let device_fd = device.as_raw_fd();
    let stdin_fd = std::io::stdin().as_raw_fd();
    let mut stdin_buf = [0u8; 64];
    let mut stdin_line = String::new();

    'recording: loop {
        let mut fds = [
            libc::pollfd {
                fd: device_fd,
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: stdin_fd,
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        let ret = unsafe { libc::poll(fds.as_mut_ptr(), 2, 500) };
        if ret < 0 {
            if unsafe { *libc::__errno_location() } == libc::EINTR {
                continue;
            }
            eprintln!("poll Fehler");
            break;
        }

        // Control commands run on every loop turn - including poll timeouts -
        // so `stop_recording` flushes even while the device is idle.
        while let Ok(command) = runtime.try_recv_command() {
            match command {
                ControlCommand::SetFilterMode { mode, .. } => {
                    warn!(%mode, "filter mode command - only honored in --filter mode");
                }
                ControlCommand::TrainProfile(name) => {
                    warn!(name, "remote training command - not yet wired");
                }
                ControlCommand::StartRecording { .. } => {
                    warn!("already recording - StartRecording ignored");
                }
                ControlCommand::StopRecording => break 'recording,
                ControlCommand::SetRecordLabel(new_label) => {
                    session.set_label(new_label);
                }
                ControlCommand::SetCompletion(config) => {
                    engine.set_completion(config);
                    runtime.set_completion(config);
                    println!(
                        "\nCompletion: mode={} radius={} max-missing={} persist={}",
                        config.mode, config.radius_factor, config.max_missing, config.persist
                    );
                }
                other => apply_registration(&mut runtime, &mut engine, other),
            }
        }

        // A stdin line marks the next LABEL_FRAME_COUNT frames; an empty line
        // or "q" ends the recording cleanly.
        if fds[1].revents & libc::POLLIN != 0 {
            let n =
                unsafe { libc::read(stdin_fd, stdin_buf.as_mut_ptr() as *mut _, stdin_buf.len()) };
            if n > 0 {
                for &byte in stdin_buf.iter().take(n as usize) {
                    match byte {
                        b'\n' | b'\r' => {
                            let text = stdin_line.trim().to_string();
                            stdin_line.clear();
                            if text.is_empty() || text == "q" {
                                break 'recording;
                            }
                            session.mark_label(RecordLabel::new(&text), LABEL_FRAME_COUNT);
                            println!(
                                "\nLabel \"{text}\" für die nächsten {LABEL_FRAME_COUNT} Frames"
                            );
                        }
                        _ => stdin_line.push(byte as char),
                    }
                }
            }
        }

        if fds[0].revents & libc::POLLIN != 0 {
            let events = match device.fetch_events() {
                Ok(events) => events,
                Err(e) => {
                    eprintln!("Lesefehler: {e}");
                    break;
                }
            };

            for raw in events {
                session.record_event(&raw, session_start.elapsed().as_secs_f64());
                let Some(engine_frame) = engine.process_with_diagnostics(&raw) else {
                    continue;
                };
                frame_no += 1;
                runtime.publish(&engine_frame, engine.tracked());

                session.record_frame(
                    FrameNumber::new(frame_no),
                    Milliseconds::from(session_start.elapsed()),
                    &engine_frame,
                    engine.tracked(),
                );
                runtime.set_recording_status(Some(status_of_session(&session)));

                let active = engine.tracked();
                print!("\x1b[2J\x1b[H");
                println!(
                    "Frame {frame_no} | {} Touch-Punkt(e) | {} Figur(en) | Label: {}",
                    engine.touch_count(),
                    active.len(),
                    session.label().map(|l| l.as_str()).unwrap_or("-"),
                );
                for t in active {
                    let deg = t.rotation_rad.to_degrees();
                    println!(
                        "  {} (id={}): center=({:.0}, {:.0}) rotation={:.1}°",
                        t.pattern_name, t.pattern_id, t.center.x, t.center.y, deg
                    );
                }
                std::io::stdout().flush().ok();
            }
        }
    }

    runtime.set_recording_status(None);
    let recorded = session.recorded_frames();
    let frame_drops = session.dropped_frames();
    let raw_drops = session.dropped_events();
    session.finish().into_diagnostic()?;
    println!("\n--- Aufzeichnung beendet ---");
    println!("Frames: {frame_no} (aufgezeichnet: {recorded}, gedroppt: {frame_drops})");
    println!("Raw-Events gedroppt: {raw_drops}");
    Ok(())
}

/// Training mode: waits for a stable set of >=3 touch points (same count over
/// [`STABLE_THRESHOLD`] frames), then saves the profile on Enter.
pub fn run_train(
    path: &Path,
    name: &str,
    tolerance: impl Into<Millimeters>,
    profiles_dir: Option<&str>,
    device_profiles: Option<&Path>,
) -> Result<()> {
    let tolerance = tolerance.into();
    let mut device = TouchDevice::open(path).into_diagnostic()?;

    println!("Trainingsmodus: {} - Figur \"{}\"", path.display(), name);
    if let Some(dev_name) = device.name() {
        println!("Gerät: {dev_name}");
    }
    println!("Lege die Figur auf die Touchfläche und halte sie still.");
    println!("Drücke Enter zum Erfassen, Ctrl+C zum Abbrechen.\n");

    let mut engine = Engine::new(Vec::new());
    if let Some(cal) = effective_calibration(&device, device_profiles) {
        println!(
            "Kalibrierung: X={:.1} units/mm, Y={:.1} units/mm",
            cal.x_res, cal.y_res
        );
        engine.set_calibration(cal);
    } else {
        eprintln!(
            "Warnung: Keine Kalibrierung vom Gerät lesbar. Profil wird in Rohkoordinaten gespeichert."
        );
    }

    const STABLE_THRESHOLD: u32 = 10;
    let mut stable_count: Option<usize> = None;
    let mut stable_frames: u32 = 0;

    // Enter detection via poll() - wait on device fd and stdin simultaneously.
    let device_fd = device.as_raw_fd();
    let stdin_fd = std::io::stdin().as_raw_fd();

    let mut captured: Option<Vec<Vec2<Millimeters>>> = None;
    let mut stdin_buf = [0u8; 64];
    let mut enter_pressed = false;

    loop {
        // Poll both fds (device + stdin), 500 ms timeout.
        let mut fds = [
            libc::pollfd {
                fd: device_fd,
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: stdin_fd,
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        let ret = unsafe { libc::poll(fds.as_mut_ptr(), 2, 500) };
        if ret < 0 {
            if unsafe { *libc::__errno_location() } == libc::EINTR {
                continue;
            }
            eprintln!("poll Fehler");
            break;
        }

        // Check stdin.
        if fds[1].revents & libc::POLLIN != 0 {
            let n =
                unsafe { libc::read(stdin_fd, stdin_buf.as_mut_ptr() as *mut _, stdin_buf.len()) };
            if n > 0 {
                for &byte in stdin_buf.iter().take(n as usize) {
                    if byte == b'\n' || byte == b'\r' {
                        enter_pressed = true;
                    }
                }
            }
        }

        // Process device events (non-blocking).
        if fds[0].revents & libc::POLLIN != 0 {
            let events = match device.fetch_events() {
                Ok(events) => events,
                Err(e) => {
                    eprintln!("Lesefehler: {e}");
                    break;
                }
            };

            for raw in events {
                if engine.process(&raw).is_some() {
                    let touches = engine.touches();
                    let count = touches.len();

                    // Stability check.
                    if stable_count == Some(count) && count >= 3 {
                        stable_frames += 1;
                    } else {
                        stable_count = Some(count);
                        stable_frames = 0;
                    }

                    print!("\x1b[2J\x1b[H");
                    println!("Trainingsmodus: \"{}\" | {} Touch-Punkt(e)", name, count);
                    if count >= 3 {
                        if stable_frames >= STABLE_THRESHOLD {
                            println!("STABIL - Drücke Enter zum Speichern");
                        } else {
                            println!("Stabilisiere... {}/{}", stable_frames, STABLE_THRESHOLD);
                        }
                        for (i, t) in touches.iter().enumerate() {
                            println!("  Punkt {i}: ({:.1}, {:.1}) mm", t.pos.x, t.pos.y);
                        }
                    } else {
                        println!("Warte auf mindestens 3 Touch-Punkte...");
                    }
                    std::io::stdout().flush().ok();

                    if enter_pressed && count >= 3 && stable_frames >= STABLE_THRESHOLD {
                        captured = Some(touches.iter().map(|t| t.pos).collect());
                        break;
                    }
                }
            }
        }

        // Check Enter even without a device event (points may already be stable).
        if enter_pressed && captured.is_none() {
            let touches = engine.touches();
            let count = touches.len();
            if count >= 3 && stable_frames >= STABLE_THRESHOLD {
                captured = Some(touches.iter().map(|t| t.pos).collect());
            }
        }

        if captured.is_some() {
            break;
        }
    }

    if let Some(points) = captured {
        let profile = ProfileFile::from_points(name, &points, tolerance);

        let dir = profiles_dir.unwrap_or("profiles");
        std::fs::create_dir_all(dir).ok();
        let filename = format!("{}/{}.toml", dir, name.to_lowercase().replace(' ', "-"));
        let filepath = PathBuf::from(&filename);

        println!("\nProfil gespeichert: {}", filepath.display());
        println!("  Name: {}", profile.name);
        println!("  Punkte: {}", profile.num_points);
        println!(
            "  Abstände ({}): {:?}",
            profile.sorted_sides.len(),
            profile
                .sorted_sides
                .iter()
                .map(|d| d.as_f64())
                .collect::<Vec<_>>()
        );
        println!("  Toleranz: {}", profile.tolerance);

        match profile.save(&filepath) {
            Ok(()) => {
                println!("\nErfolgreich! Profil kann mit --live oder --replay verwendet werden.")
            }
            Err(e) => eprintln!("Fehler beim Speichern: {e}"),
        }
    } else {
        eprintln!("\nTraining abgebrochen.");
    }
    Ok(())
}

/// Resolves the device argument: an explicit `--device` path wins;
/// otherwise auto-detect via `vendor:product` against the device registry
/// (builtin + `--device-profiles` dir) - succeeds only when exactly one
/// known touchscreen is attached.
pub fn resolve_device(device: Option<String>, profiles_dir: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = device {
        return Ok(PathBuf::from(path));
    }
    let mut registry = tangible_objects_model::DeviceRegistry::builtin();
    if let Some(dir) = profiles_dir {
        registry = registry.with_dir(dir);
    }
    let entry = tangible_objects_engine::TouchDevice::detect(&registry).into_diagnostic()?;
    let profile = registry
        .find_by_ids(entry.vendor, entry.product)
        .expect("detected entry matches a profile");
    println!(
        "Gerät erkannt: {} ({}) → {}",
        profile.name,
        entry.name,
        entry.path.display()
    );
    Ok(entry.path)
}

/// Converts a `.evtest` recording into the raw event stream of another
/// touchscreen profile (TRAINING.md device conversion).
///
/// `target` resolves against the device registry (builtin + `--device-profiles`
/// dir) or as a TOML file path; the output lands next to the input as
/// `<stem>.to-<profile-name>.evtest`.
pub fn run_convert_device(input: &Path, target: &str, profiles_dir: Option<&Path>) -> Result<()> {
    let mut registry = tangible_objects_model::DeviceRegistry::builtin();
    if let Some(dir) = profiles_dir {
        registry = registry.with_dir(dir);
    }
    for (path, error) in &registry.errors {
        eprintln!(
            "Warnung: Geräteprofil {} konnte nicht geladen werden: {error}",
            path.display()
        );
    }
    let profile = registry.resolve(target).ok_or_else(|| {
        miette::miette!(
            "Geräteprofil '{target}' nicht gefunden (bekannt: {})",
            registry
                .profiles
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    let out = input.with_extension(format!(
        "to-{}.evtest",
        profile.name.replace(' ', "-").to_lowercase()
    ));
    let converter = tangible_objects_engine::DeviceConverter::new(profile.clone());
    let stats = converter.convert(input, &out).into_diagnostic()?;
    println!(
        "Konvertiert: {} → {} ({} Events: {} übernommen, {} gedroppt, {} geclampt)",
        input.display(),
        out.display(),
        stats.events,
        stats.kept,
        stats.dropped,
        stats.clamped
    );
    Ok(())
}

/// Labeled dataset export of a `.jsonl` recording (TRAINING.md 3.3).
///
/// Writes `<stem>.dataset.csv` next to the recording - one CSV row per
/// candidate of every labeled frame (normalized deviation features +
/// ground-truth target) for external ML tooling.
pub fn run_export_dataset(path: &Path) -> Result<()> {
    let recording = tangible_objects_analysis::Recording::load(path).into_diagnostic()?;
    let out = path.with_extension("dataset.jsonl");
    let dataset = tangible_objects_analysis::Dataset::of(&recording);
    dataset.write_jsonl(&out).into_diagnostic()?;
    println!(
        "Dataset: {} ({} rows, {} features)",
        out.display(),
        dataset.rows.len(),
        dataset.feature_width
    );
    if dataset.rows.is_empty() {
        println!("  (keine gelabelten Frames - mit --record-label aufzeichnen)");
    }
    Ok(())
}

/// Offline analysis of a `.jsonl` recording (TRAINING.md phase 2).
///
/// Prints utilization, tolerance suggestions, flappy-contact metrics and the
/// collision matrix for the loaded frames.
pub fn run_analyze(path: &Path) -> Result<()> {
    let recording = tangible_objects_analysis::Recording::load(path).into_diagnostic()?;
    println!(
        "Analyse: {} ({} Frames)\n",
        path.display(),
        recording.frames().len()
    );
    print!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        tangible_objects_analysis::UtilizationReport::analyze(&recording),
        tangible_objects_analysis::DeviationReport::analyze(&recording),
        tangible_objects_analysis::ToleranceReport::analyze(&recording),
        tangible_objects_analysis::FlappyReport::analyze(&recording),
        tangible_objects_analysis::CollisionReport::analyze(&recording),
        tangible_objects_analysis::ScoringReport::analyze(&recording),
        tangible_objects_analysis::MlReport::analyze(&recording),
    );
    Ok(())
}

/// Prints the pairwise profile-similarity report for a profile
/// directory - the phantom-collision surface (which reduced contact
/// sets of one figure can satisfy another figure's fingerprint).
pub fn run_similarity(dir: &Path) -> Result<()> {
    let loaded = tangible_objects_model::ProfileFile::load_dir(dir);
    for (file, error) in &loaded.errors {
        eprintln!("{}: {error}", file.display());
    }
    let patterns = loaded.into_patterns();
    println!(
        "Aehnlichkeits-Analyse: {} ({} Profile)\n",
        dir.display(),
        patterns.len()
    );
    println!(
        "{}",
        tangible_objects_analysis::SimilarityReport::analyze(&patterns)
    );
    Ok(())
}

/// Lists all `/dev/input/eventX` devices with names and multitouch markers.
pub fn list_devices() {
    let entries = TouchDevice::enumerate();
    if entries.is_empty() {
        eprintln!("Keine Eingabegeräte in /dev/input/ gefunden.");
        return;
    }
    for entry in entries {
        println!("{entry}");
    }
}
