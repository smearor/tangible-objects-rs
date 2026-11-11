//! `RegisterSpellInput` - the `registerSpell` input object, a wire
//! DTO of [`Spell`] (SPELLS.md).

use async_graphql::Error;
use async_graphql::Result;
use tangible_objects_model::FingerCount;
use tangible_objects_model::Millimeters;
use tangible_objects_model::Spell;
use tangible_objects_model::SpellArming;
use tangible_objects_model::SpellName;
use tangible_objects_model::StrokePolyline;
use tangible_objects_model::Vec2;

use super::spell_arming_input::SpellArmingInput;

/// The corridor default in millimeters when a registration leaves it
/// unset - matches the `spells.d` example value.
const DEFAULT_SPELL_CORRIDOR_MM: f64 = 12.0;

/// Input for `registerSpell`: a gesture defined by per-finger
/// template strokes.
#[derive(async_graphql::InputObject)]
pub(crate) struct RegisterSpellInput {
    /// Spell name (identity for unregister and the runtime store).
    name: SpellName,
    /// Max per-stroke deviation in millimeters; `null` = 12 mm.
    corridor: Option<Millimeters>,
    /// Whether the gesture may be drawn larger/smaller than the
    /// template; `null` = true.
    scale_invariant: Option<bool>,
    /// Whether the spell matches; `null` = true.
    enabled: Option<bool>,
    /// Per-finger template strokes - `strokes[i]` is finger i's flat
    /// polyline `[x0, y0, x1, y1, ...]` in millimeters (same shape as
    /// `registerPattern.points`). The count defines `fingers`.
    strokes: Vec<Vec<Millimeters>>,
    /// How the gesture is initiated; `null` = `free`.
    arming: Option<SpellArmingInput>,
}

impl TryFrom<&RegisterSpellInput> for Spell {
    type Error = Error;

    /// Projects the wire input into the domain spell - validates the
    /// stroke count and point arity.
    fn try_from(input: &RegisterSpellInput) -> Result<Self> {
        if input.strokes.is_empty() || input.strokes.len() > 5 {
            return Err(Error::new("spell needs 1..=5 stroke templates"));
        }
        let mut strokes = Vec::with_capacity(input.strokes.len());
        for stroke in &input.strokes {
            if stroke.len() < 2 {
                return Err(Error::new("stroke needs at least 2 points"));
            }
            if stroke.len() % 2 != 0 {
                return Err(Error::new("stroke point list must be flat [x, y] pairs"));
            }
            let (pairs, _) = stroke.as_chunks::<2>();
            let points = pairs
                .iter()
                .map(|pair| Vec2::from_mm(pair[0], pair[1]))
                .collect();
            strokes.push(StrokePolyline::new(points));
        }
        Ok(Self {
            name: input.name.clone(),
            fingers: FingerCount::new(strokes.len()),
            corridor: input
                .corridor
                .unwrap_or(Millimeters::new(DEFAULT_SPELL_CORRIDOR_MM)),
            scale_invariant: input.scale_invariant.unwrap_or(true),
            enabled: input.enabled.unwrap_or(true),
            strokes,
            arming: input
                .arming
                .as_ref()
                .map(SpellArming::try_from)
                .transpose()?
                .unwrap_or(SpellArming::Free),
        })
    }
}
