use std::path::Path;
use std::path::PathBuf;

#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

use super::Field;
use super::FieldName;
use super::FieldSetName;
use super::WindowAnchor;
use crate::FrameCount;
use crate::TouchPoint;
use crate::Vec2;

/// Default debounce stability for field sets (2 frames).
pub fn default_stability() -> FrameCount {
    FrameCount::new(2)
}

/// A named set of occupancy fields - the application-supplied board.
///
/// Loaded from a `.toml` field-set file:
///
/// ```toml
/// name = "chess"
/// stability = 2
///
/// [window]          # optional: fields are window-relative
/// x_offset = 50.0
/// y_offset = 30.0
///
/// [[fields]]
/// name = "a1"
/// x_min = 0.0
/// y_min = 0.0
/// x_max = 100.0
/// y_max = 100.0
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "FieldSet",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct FieldSet {
    /// Human-readable set name - the namespace of its fields.
    pub name: FieldSetName,
    /// Optional window anchor: field coordinates are interpreted
    /// relative to the application window instead of screen-absolute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<WindowAnchor>,
    /// Consecutive frames required to confirm a count transition
    /// (debouncing, symmetric for place and lift). Default 2.
    #[serde(default = "default_stability")]
    pub stability: FrameCount,
    /// The occupancy cells of this set. Must be mutually disjoint -
    /// see [`Self::is_disjoint`].
    #[serde(rename = "fields")]
    pub fields: Vec<Field>,
    /// Whether contacts inside this set's fields are suppressed by the
    /// filter proxy (they never reach the OS). Default `false` -
    /// occupancy is informational only.
    #[serde(default)]
    pub suppress: bool,
}

impl FieldSet {
    /// Whether `touch` lies inside `field` - evaluated in the field's
    /// coordinate space: window-relative when the set carries a
    /// `window` anchor, screen-absolute otherwise.
    pub fn field_contains(&self, field: &Field, touch: &TouchPoint) -> bool {
        let local = match &self.window {
            Some(anchor) => anchor.to_local(&touch.pos),
            None => Vec2::new(touch.pos.x, touch.pos.y),
        };
        field.region.contains(&local)
    }

    /// Whether all fields are mutually disjoint (no overlapping cells).
    /// Registration of an overlapping set is rejected.
    pub fn is_disjoint(&self) -> bool {
        self.overlap_pair().is_none()
    }

    /// The first overlapping field pair, if any - names the offenders
    /// for a rejection message ("why"), not just a boolean.
    pub fn overlap_pair(&self) -> Option<(&FieldName, &FieldName)> {
        for (i, a) in self.fields.iter().enumerate() {
            for b in &self.fields[i + 1..] {
                if a.region.overlaps(&b.region) {
                    return Some((&a.name, &b.name));
                }
            }
        }
        None
    }

    /// Loads a field set from a `.toml` file.
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    }

    /// Loads all `.toml` field-set files from a directory, sorted by
    /// filename. Unreadable or invalid files are skipped and returned
    /// in the error list.
    pub fn load_dir(dir: &Path) -> (Vec<Self>, Vec<(PathBuf, std::io::Error)>) {
        let mut sets = Vec::new();
        let mut errors = Vec::new();
        let Ok(entries) = std::fs::read_dir(dir) else {
            return (sets, errors);
        };
        let mut files: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
            .collect();
        files.sort();
        for file in files {
            match Self::load(&file) {
                Ok(set) => sets.push(set),
                Err(error) => errors.push((file, error)),
            }
        }
        (sets, errors)
    }
}

#[cfg(test)]
mod tests {
    use super::FieldSet;
    use crate::Vec2;

    fn parse(toml: &str) -> Result<FieldSet, toml::de::Error> {
        toml::from_str(toml)
    }

    #[test]
    fn toml_parses_fields_and_defaults() -> Result<(), Box<dyn std::error::Error>> {
        let set = parse(
            r#"
            name = "chess"

            [[fields]]
            name = "a1"
            x_min = 0.0
            y_min = 0.0
            x_max = 100.0
            y_max = 100.0
            "#,
        )?;
        assert_eq!(set.name.as_str(), "chess");
        assert_eq!(set.stability.as_u64(), 2);
        assert!(set.window.is_none());
        assert_eq!(set.fields.len(), 1);
        assert!(set.is_disjoint());
        Ok(())
    }

    #[test]
    fn window_anchor_and_normalized_fields() -> Result<(), Box<dyn std::error::Error>> {
        let set = parse(
            r#"
            name = "board"
            [window]
            x_offset = 50.0
            y_offset = 30.0
            width = 400.0
            height = 400.0

            [[fields]]
            name = "cell"
            x_min = 0.0
            y_min = 0.0
            x_max = 0.5
            y_max = 0.5
            "#,
        )?;
        let anchor = set.window.ok_or("window anchor expected")?;
        let local = anchor.to_local(&Vec2::new(250.0, 230.0));
        assert_eq!(local, Vec2::new(0.5, 0.5));
        assert!(set.fields[0].region.contains(&local));
        let outside = anchor.to_local(&Vec2::new(350.0, 230.0));
        assert!(!set.fields[0].region.contains(&outside));
        Ok(())
    }

    #[test]
    fn overlapping_fields_are_detected() -> Result<(), Box<dyn std::error::Error>> {
        let set = parse(
            r#"
            name = "bad"
            [[fields]]
            name = "a"
            x_min = 0.0
            y_min = 0.0
            x_max = 100.0
            y_max = 100.0
            [[fields]]
            name = "b"
            x_min = 50.0
            y_min = 50.0
            x_max = 150.0
            y_max = 150.0
            "#,
        )?;
        assert!(!set.is_disjoint());
        Ok(())
    }
}
