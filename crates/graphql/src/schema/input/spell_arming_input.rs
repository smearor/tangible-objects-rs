//! `SpellArmingInput` - the spell arming selector wire DTO of
//! [`SpellArming`] (SPELLS.md's Einläutung types).

use async_graphql::Error;
use async_graphql::Result;
use tangible_objects_model::PatternName;
use tangible_objects_model::Region;
use tangible_objects_model::SpellArming;

use super::region_input::RegionInput;

/// The arming type - maps kebab-case spell TOML values.
#[derive(async_graphql::Enum, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpellArmingKind {
    /// Freeform stroke - always armed (legacy/fallback behavior).
    Free,
    /// A spell-dedicated area; every stroke point must stay inside.
    ExclusiveZone,
    /// A small region the episode's first contact must start inside.
    StartSpot,
    /// A short simultaneous five-contact tap arms the next episode.
    FiveFingerTap,
    /// A tracked tangible of `pattern` acts as the wand.
    TangibleChord,
}

/// Input for a spell's arming configuration - `region` is required
/// for `EXCLUSIVE_ZONE`/`START_SPOT`, `pattern` for `TANGIBLE_CHORD`.
#[derive(async_graphql::InputObject)]
pub(crate) struct SpellArmingInput {
    /// Which trigger arms the spell.
    kind: SpellArmingKind,
    /// Bounding box for `EXCLUSIVE_ZONE` / `START_SPOT` arming.
    region: Option<RegionInput>,
    /// Tangible pattern for `TANGIBLE_CHORD` arming; `null` = any
    /// tangible may act as the wand.
    pattern: Option<PatternName>,
}

impl TryFrom<&SpellArmingInput> for SpellArming {
    type Error = Error;

    /// Projects the wire input into the domain arming - validates
    /// that `region`/`pattern` are present for the kinds that need
    /// them.
    fn try_from(input: &SpellArmingInput) -> Result<Self> {
        let region = || -> Result<Region> {
            input
                .region
                .as_ref()
                .map(Region::from)
                .ok_or_else(|| Error::new("arming kind requires `region`"))
        };
        match input.kind {
            SpellArmingKind::Free => Ok(SpellArming::Free),
            SpellArmingKind::ExclusiveZone => Ok(SpellArming::ExclusiveZone { region: region()? }),
            SpellArmingKind::StartSpot => Ok(SpellArming::StartSpot { region: region()? }),
            SpellArmingKind::FiveFingerTap => Ok(SpellArming::FiveFingerTap),
            SpellArmingKind::TangibleChord => Ok(SpellArming::TangibleChord {
                pattern: input.pattern.clone(),
            }),
        }
    }
}
