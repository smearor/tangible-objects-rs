//! `spell` - multi-finger gesture recognition (SPELLS.md):
//! per-finger template polylines matched via discrete-Fréchet
//! corridor after translation/scale normalization.

mod arming;
mod definition;
mod finger_count;
mod matched;
mod name;
mod stroke_polyline;

pub use arming::SpellArming;
pub use definition::REFERENCE_EXTENT_MM;
pub use definition::Spell;
pub use finger_count::FingerCount;
pub use matched::SpellMatched;
pub use name::SpellName;
pub use stroke_polyline::STROKE_SAMPLES;
pub use stroke_polyline::StrokePolyline;
