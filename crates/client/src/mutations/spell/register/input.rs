//! `RegisterSpellInput` - the cynic input object for
//! `registerSpell`, a wire DTO of [`Spell`].

use cynic::InputObject;
use smearor_commons_geometry::Millimeters;
use tangible_objects_model::Spell;
use tangible_objects_model::SpellName;

use crate::mutations::spell::register::arming_input::SpellArmingInput;

/// The `RegisterSpellInput` input object - wire DTO of [`Spell`].
#[derive(InputObject)]
#[cynic(
    graphql_type = "RegisterSpellInput",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    rename_all = "camelCase"
)]
pub(crate) struct RegisterSpellInput {
    /// Spell name (identity for unregister and the runtime store).
    name: SpellName,
    /// Max per-stroke deviation in millimeters; `None` = daemon
    /// default.
    corridor: Option<Millimeters>,
    /// Whether the gesture may be drawn larger/smaller than the
    /// template; `None` = daemon default.
    scale_invariant: Option<bool>,
    /// Whether the spell matches; `None` = daemon default.
    enabled: Option<bool>,
    /// Per-finger template strokes - `strokes[i]` is finger i's
    /// flat polyline `[x0, y0, x1, y1, ...]` in millimeters.
    strokes: Vec<Vec<Millimeters>>,
    /// How the gesture is initiated; `None` = `free`.
    arming: Option<SpellArmingInput>,
}

impl From<&Spell> for RegisterSpellInput {
    /// Projects the spell into the wire input shape - every
    /// [`StrokePolyline`] flattens its `Vec2` points into the
    /// `[x0, y0, x1, y1, ...]` wire form.
    fn from(spell: &Spell) -> Self {
        Self {
            name: spell.name.clone(),
            corridor: Some(spell.corridor),
            scale_invariant: Some(spell.scale_invariant),
            enabled: Some(spell.enabled),
            strokes: spell
                .strokes
                .iter()
                .map(|stroke| {
                    stroke
                        .points
                        .iter()
                        .flat_map(|point| [Millimeters::new(point.x), Millimeters::new(point.y)])
                        .collect()
                })
                .collect(),
            arming: Some((&spell.arming).into()),
        }
    }
}
