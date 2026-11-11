//! Dead zones - blacklisted surface positions.
//!
//! Some panels report ghost contacts at fixed positions: stuck MT
//! events, damaged sensor areas or edge glitches. A [`DeadZone`]
//! blacklists a circular spot so contacts inside its `radius` are
//! dropped at ingest - they never reach smoothing, matching, fields,
//! sets or metrics. Zones are managed at runtime through the
//! `Registration::DeadZone` command (session or persistent lifetime),
//! mirroring field-set registration.

use smearor_commons_geometry::Millimeters;
mod name;
mod zone;
mod zones;

/// Interface default for a dead zone's suppression radius: 10 mm.
/// Used when a registration omits the radius (`null`/`0` on the wire).
pub const DEFAULT_DEAD_ZONE_RADIUS: Millimeters = Millimeters::new(10.0);

pub use name::DeadZoneName;
pub use zone::DeadZone;
pub use zones::DeadZones;
