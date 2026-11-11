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

#[cfg(feature = "cynic")]
impl cynic::QueryFragment for DeadZone {
    type SchemaType = schema::DeadZone;
    type VariablesFields = ();
    const TYPE: Option<&'static str> = Some("DeadZone");

    /// Selects `name`, `center { x y }` and `radius` - the response
    /// still decodes through `Deserialize`, the fields are selection
    /// markers only.
    fn query(
        builder: cynic::queries::SelectionBuilder<'_, Self::SchemaType, Self::VariablesFields>,
    ) {
        let mut builder = builder;
        let _ = builder.select_field::<schema::__fields::DeadZone::name, schema::DeadZoneName>();
        let mut center =
            builder.select_field::<schema::__fields::DeadZone::center, schema::Point>();
        let mut point = center.select_children::<()>();
        let _ = point.select_field::<schema::__fields::Point::x, f64>();
        let _ = point.select_field::<schema::__fields::Point::y, f64>();
        let _ = builder.select_field::<schema::__fields::DeadZone::radius, schema::Millimeters>();
    }
}
