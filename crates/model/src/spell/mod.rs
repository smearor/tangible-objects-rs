//! `spell` - multi-finger gesture recognition (SPELLS.md):
//! per-finger template polylines matched via discrete-Fréchet
//! corridor after translation/scale normalization.

mod arming;
mod finger_count;
mod name;
mod spell;
mod stroke_polyline;

pub use arming::SpellArming;
pub use finger_count::FingerCount;
pub use name::SpellName;
pub use spell::REFERENCE_EXTENT_MM;
pub use spell::Spell;
pub use stroke_polyline::STROKE_SAMPLES;
pub use stroke_polyline::StrokePolyline;
