use std::path::Path;
use std::path::PathBuf;

#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

use super::DeadZoneName;
use crate::Millimeters;
use crate::Vec2;

/// A blacklisted surface position: contacts inside `radius` of `center`
/// are dropped at ingest.
///
/// Ghost contacts reported by stuck hardware events or damaged sensor
/// spots never reach smoothing, matching, fields, sets or metrics.
/// Registered and removed at runtime like field sets
/// (`Registration::DeadZone`, session or persistent lifetime); the
/// interface default radius is 10 mm.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "DeadZone",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct DeadZone {
    /// Identity of the zone - addresses it for unregister and the
    /// runtime store.
    pub name: DeadZoneName,
    /// Center of the suppressed spot in millimeters.
    pub center: Vec2<Millimeters>,
    /// Suppression radius in millimeters around `center`.
    pub radius: Millimeters,
}

impl DeadZone {
    /// Creates a dead zone of `radius` millimeters around `center`.
    pub fn new(name: DeadZoneName, center: Vec2<Millimeters>, radius: Millimeters) -> Self {
        Self {
            name,
            center,
            radius,
        }
    }

    /// Whether `pos` falls inside the zone's suppression radius.
    pub fn contains(&self, pos: &Vec2<Millimeters>) -> bool {
        self.center.distance(pos) <= self.radius
    }

    /// Loads a single dead-zone `.toml` file.
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    }

    /// Loads all `.toml` dead-zone files from a directory, sorted by
    /// filename. Unreadable or invalid files are skipped and returned
    /// in the error list.
    pub fn load_dir(dir: &Path) -> (Vec<Self>, Vec<(PathBuf, std::io::Error)>) {
        let mut zones = Vec::new();
        let mut errors = Vec::new();
        let Ok(entries) = std::fs::read_dir(dir) else {
            return (zones, errors);
        };
        let mut files: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
            .collect();
        files.sort();
        for file in files {
            match Self::load(&file) {
                Ok(zone) => zones.push(zone),
                Err(error) => errors.push((file, error)),
            }
        }
        (zones, errors)
    }
}
