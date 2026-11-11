//! Runtime store for `Lifetime::Persistent` registrations -
//! `~/.config/tangible-objects/profiles.d/` resp. `fields.d/`.

use std::path::Path;
use std::path::PathBuf;

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

/// Deletes a persisted registration file, if it exists.
pub(crate) fn unpersist(store_kind: &str, name: &str) {
    if let Some(path) = runtime_store_dir(store_kind).map(|dir| dir.join(format!("{name}.toml")))
        && path.exists()
        && let Err(e) = std::fs::remove_file(&path)
    {
        warn!(error = %e, path = %path.display(), "unpersist failed");
    }
}
