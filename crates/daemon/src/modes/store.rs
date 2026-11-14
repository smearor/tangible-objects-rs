//! Runtime store for `Lifetime::Persistent` registrations -
//! `~/.config/tangible-objects/profiles.d/` resp. `fields.d/`.

use std::path::Path;
use std::path::PathBuf;

use tangible_objects_engine::Engine;
use tangible_objects_model::ProfileFile;
use tangible_objects_model::Registration;
use tracing::info;
use tracing::warn;

use super::setup::completion_config_path;

/// Runtime store for `Lifetime::Persistent` registrations:
/// `~/.config/tangible-objects/profiles.d/` resp. `fields.d/`.
pub(crate) fn runtime_store_dir(kind: &str) -> Option<PathBuf> {
    completion_config_path()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .map(|base| base.join(kind))
}

/// Persists a registered profile/field set into the runtime store.
pub(crate) fn persist_registration(registration: &Registration) {
    let Some(store) = runtime_store_dir(match registration {
        Registration::Pattern(_) => "profiles.d",
        Registration::FieldSet(_) => "fields.d",
        Registration::DeadZone(_) => "dead_zones.d",
        Registration::Spell(_) => "spells.d",
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
        Registration::DeadZone(zone) => (
            zone.name.as_str().to_string(),
            std::fs::create_dir_all(&store).and_then(|_| {
                std::fs::write(
                    store.join(format!("{}.toml", zone.name)),
                    toml::to_string_pretty(zone).unwrap_or_default(),
                )
            }),
        ),
        Registration::Spell(spell) => (
            spell.name.as_str().to_string(),
            std::fs::create_dir_all(&store)
                .and_then(|_| spell.save(&store.join(format!("{}.toml", spell.name)))),
        ),
    };
    match result {
        Ok(()) => info!(name, "registration persisted"),
        Err(e) => warn!(error = %e, name, "persist failed"),
    }
}

/// Rewrites each loaded profile's `enabled`/`color` from the live
/// engine state - the profiles card's Save button persists the
/// runtime overrides (`SetProfileEnabled`/`SetProfileColor`) back to
/// disk. Session-only patterns without a profile file are skipped.
/// Returns the number of rewritten files.
pub(crate) fn save_profile_states(engine: &Engine, profiles_dir: Option<&Path>) -> usize {
    let dir = profiles_dir.unwrap_or_else(|| Path::new("profiles"));
    let mut loaded = ProfileFile::load_dir(dir);
    if let Some(store) = runtime_store_dir("profiles.d") {
        let mut stored = ProfileFile::load_dir(&store);
        loaded.profiles.append(&mut stored.profiles);
        loaded.errors.append(&mut stored.errors);
    }
    for (path, error) in &loaded.errors {
        warn!(path = %path.display(), %error, "profile unreadable - skipped on save");
    }
    let patterns = engine.patterns();
    let mut written = 0usize;
    for (path, mut profile) in loaded.profiles {
        let Some(pattern) = patterns.iter().find(|pattern| pattern.name == profile.name) else {
            continue;
        };
        if profile.enabled == pattern.enabled && profile.color == pattern.color {
            continue;
        }
        profile.enabled = pattern.enabled;
        profile.color = pattern.color;
        match profile.save(&path) {
            Ok(()) => {
                written += 1;
                info!(path = %path.display(), enabled = pattern.enabled, "profile state saved");
            }
            Err(error) => warn!(%error, path = %path.display(), "profile save failed"),
        }
    }
    written
}

/// Deletes a persisted registration file, if it exists.
pub(crate) fn unpersist(store_kind: &str, name: &str) {
    if let Some(path) = runtime_store_dir(store_kind).map(|dir| dir.join(format!("{name}.toml")))
        && path.exists()
        && let Err(e) = std::fs::remove_file(&path)
    {
        warn!(error = %e, path = %path.display(), "unpersist failed");
    }
}

#[cfg(test)]
mod tests {
    use smearor_commons_geometry::Vec2;
    use tangible_objects_engine::Engine;
    use tangible_objects_model::FigureColor;
    use tangible_objects_model::PatternId;
    use tangible_objects_model::ProfileFile;

    use super::save_profile_states;

    fn temp_dir(test_name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "robofigure-store-{test_name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or_default()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn save_profile_states_writes_enabled_and_color_back() {
        let dir = temp_dir("enabled");
        let points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        let mut file = ProfileFile::from_points("fig", &points, 5.0);
        file.enabled = false;
        file.save(&dir.join("fig.toml")).expect("seed profile");
        let mut engine = Engine::new(vec![file.to_pattern(PatternId::new(1))]);
        let color = FigureColor::new(1.0, 0.0, 0.0);

        assert_eq!(save_profile_states(&engine, Some(&dir)), 0);
        engine.set_pattern_enabled(PatternId::new(1), true);
        engine.set_pattern_color(PatternId::new(1), color);

        assert_eq!(save_profile_states(&engine, Some(&dir)), 1);
        let loaded = ProfileFile::load_dir(&dir)
            .profiles
            .pop()
            .map(|(_, profile)| profile)
            .expect("profile reloads");
        assert!(loaded.enabled);
        assert_eq!(loaded.color, Some(color));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_profile_states_skips_session_only_patterns() {
        let dir = temp_dir("session");
        let points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 0.0),
            Vec2::new(0.0, 50.0),
        ];
        let engine = Engine::new(vec![
            ProfileFile::from_points("ghost", &points, 5.0).to_pattern(PatternId::new(1)),
        ]);

        assert_eq!(save_profile_states(&engine, Some(&dir)), 0);
        assert!(ProfileFile::load_dir(&dir).profiles.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }
}
