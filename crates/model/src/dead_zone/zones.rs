use std::ops::Deref;

use serde::Deserialize;
use serde::Serialize;

use super::DeadZone;
use super::DeadZoneName;
use crate::Millimeters;
use crate::RegistrationError;
use crate::Vec2;

/// The registered dead zones of a session.
///
/// Ingest drops every contact whose position falls inside any zone -
/// ghost spots on the panel suppress touches before they reach
/// smoothing, matching, fields, sets or metrics.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeadZones(pub Vec<DeadZone>);

impl DeadZones {
    /// Whether `pos` falls inside any registered zone's radius.
    pub fn contains(&self, pos: &Vec2<Millimeters>) -> bool {
        self.0.iter().any(|zone| zone.contains(pos))
    }

    /// Adds a zone; `Err(DuplicateDeadZone)` on name collision.
    pub fn register(&mut self, zone: DeadZone) -> Result<(), RegistrationError> {
        if self.0.iter().any(|z| z.name == zone.name) {
            return Err(RegistrationError::DuplicateDeadZone(zone.name.clone()));
        }
        self.0.push(zone);
        Ok(())
    }

    /// Removes the zone by name; `true` when one was removed.
    pub fn unregister(&mut self, name: &DeadZoneName) -> bool {
        let len = self.0.len();
        self.0.retain(|zone| zone.name != *name);
        self.0.len() != len
    }
}

impl From<Vec<DeadZone>> for DeadZones {
    /// A vector of zones IS a zone registry.
    fn from(zones: Vec<DeadZone>) -> Self {
        Self(zones)
    }
}

impl Deref for DeadZones {
    type Target = [DeadZone];

    /// Zones view their slice transparently.
    fn deref(&self) -> &[DeadZone] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::DeadZone;
    use super::DeadZoneName;
    use super::DeadZones;
    use crate::Millimeters;
    use crate::Vec2;

    fn zone(name: &str, x: f64, y: f64, radius: f64) -> DeadZone {
        DeadZone::new(
            DeadZoneName::new(name),
            Vec2::<Millimeters>::new(x, y),
            Millimeters::new(radius),
        )
    }

    /// A position inside the radius is suppressed, outside is not.
    #[test]
    fn contains_checks_the_radius() {
        let zones: DeadZones = vec![zone("edge-ghost", 100.0, 50.0, 10.0)].into();
        assert!(zones.contains(&Vec2::new(105.0, 50.0)));
        assert!(zones.contains(&Vec2::new(100.0, 50.0)));
        assert!(!zones.contains(&Vec2::new(111.0, 50.0)));
        assert!(!zones.contains(&Vec2::new(50.0, 50.0)));
        assert!(!DeadZones::default().contains(&Vec2::new(100.0, 50.0)));
    }

    /// Registering rejects duplicate names; unregister removes by name.
    #[test]
    fn register_rejects_duplicates_and_unregister_removes() {
        let mut zones = DeadZones::default();
        assert!(zones.register(zone("a", 0.0, 0.0, 10.0)).is_ok());
        assert!(zones.register(zone("a", 1.0, 1.0, 10.0)).is_err());
        assert!(zones.register(zone("b", 1.0, 1.0, 10.0)).is_ok());
        assert!(zones.unregister(&DeadZoneName::new("a")));
        assert!(!zones.unregister(&DeadZoneName::new("a")));
        assert_eq!(zones.len(), 1);
    }
}
