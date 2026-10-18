//! Runtime store for `Lifetime::Persistent` registrations -
//! `~/.config/tangible-objects/profiles.d/` resp. `fields.d/`.

use std::path::Path;
use std::path::PathBuf;

use tangible_objects_model::Registration;
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
pub(crate) fn unpersist(store_kind: &str, name: &str) {
    if let Some(path) = runtime_store_dir(store_kind).map(|dir| dir.join(format!("{name}.toml")))
        && path.exists()
        && let Err(e) = std::fs::remove_file(&path)
    {
        warn!(error = %e, path = %path.display(), "unpersist failed");
    }
}
