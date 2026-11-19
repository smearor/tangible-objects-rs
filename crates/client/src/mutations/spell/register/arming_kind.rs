//! `SpellArmingKind` - the cynic enum tagging
//! [`SpellArmingInput`](crate::mutations::spell::register::arming_input::SpellArmingInput)
//! variants.

use cynic::Enum;

/// The `SpellArmingKind` wire enum - mirrors the `SpellArming`
/// variant tags in the schema's SCREAMING_SNAKE form.
#[derive(Enum)]
#[cynic(
    graphql_type = "SpellArmingKind",
    schema_path = "../graphql/schema.graphql",
    schema_module = "tangible_objects_model::schema",
    rename_all = "SCREAMING_SNAKE_CASE"
)]
pub(crate) enum SpellArmingKind {
    /// Freeform stroke - always armed.
    Free,
    /// A spell-dedicated area; every stroke point must stay inside.
    ExclusiveZone,
    /// A small region the first contact must start inside.
    StartSpot,
    /// A short simultaneous five-contact tap arms the next episode.
    FiveFingerTap,
    /// A tracked tangible acts as the wand.
    TangibleChord,
}
