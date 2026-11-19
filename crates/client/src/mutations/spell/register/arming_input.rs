//! `SpellArmingInput` - the cynic input object for the spell's
//! arming trigger, a wire DTO of [`SpellArming`].

use cynic::InputObject;
use tangible_objects_model::PatternName;
use tangible_objects_model::SpellArming;

use crate::mutations::spell::register::arming_kind::SpellArmingKind;
use crate::mutations::spell::register::region_input::RegionInput;

/// The `SpellArmingInput` input object - the `kind` tag plus the
/// fields the armed variants carry (`region` for zone/spot arming,
/// `pattern` for the tangible wand).
#[derive(InputObject)]
#[cynic(
    graphql_type = "SpellArmingInput",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    rename_all = "camelCase"
)]
pub(crate) struct SpellArmingInput {
    /// Which trigger arms the spell.
    kind: SpellArmingKind,
    /// Bounding box for `ExclusiveZone`/`StartSpot` arming.
    region: Option<RegionInput>,
    /// Wand pattern for `TangibleChord` arming; `None` = any
    /// tangible.
    pattern: Option<PatternName>,
}

impl From<&SpellArming> for SpellArmingInput {
    /// Projects the arming enum into the tagged input shape.
    fn from(arming: &SpellArming) -> Self {
        match arming {
            SpellArming::Free => Self {
                kind: SpellArmingKind::Free,
                region: None,
                pattern: None,
            },
            SpellArming::ExclusiveZone { region } => Self {
                kind: SpellArmingKind::ExclusiveZone,
                region: Some(region.into()),
                pattern: None,
            },
            SpellArming::StartSpot { region } => Self {
                kind: SpellArmingKind::StartSpot,
                region: Some(region.into()),
                pattern: None,
            },
            SpellArming::FiveFingerTap => Self {
                kind: SpellArmingKind::FiveFingerTap,
                region: None,
                pattern: None,
            },
            SpellArming::TangibleChord { pattern } => Self {
                kind: SpellArmingKind::TangibleChord,
                region: None,
                pattern: pattern.clone(),
            },
        }
    }
}
