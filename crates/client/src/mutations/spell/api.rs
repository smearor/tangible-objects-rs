//! `register_spell()`/`unregister_spell()` methods of
//! [`GraphQLClient`].

use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::RegisterSpellResult;
use tangible_objects_model::Spell;
use tangible_objects_model::SpellArming;
use tangible_objects_model::SpellName;

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::spell::register::arming_input::SpellArmingInput;
use crate::mutations::spell::register::mutation::RegisterSpell;
use crate::mutations::spell::register::result::RegisterSpellResultFragment;
use crate::mutations::spell::register::variables::RegisterSpellVariables;
use crate::mutations::spell::train::mutation::TrainSpell;
use crate::mutations::spell::train::variables::TrainSpellVariables;
use crate::mutations::spell::unregister::mutation::UnregisterSpell;
use crate::mutations::spell::unregister::variables::UnregisterSpellVariables;

impl GraphQLClient {
    /// Registers a multi-finger gesture (`registerSpell`) - the
    /// `Spell`'s template strokes flatten into the InputObject
    /// shape. The result union carries the typed rejection, not
    /// just a message.
    pub async fn register_spell(
        &self,
        spell: &Spell,
        owner: Option<&Owner>,
        lifetime: Lifetime,
    ) -> Result<RegisterSpellResult, ClientError> {
        let fragment: RegisterSpellResultFragment = self
            .call_mutation::<RegisterSpell, _, _>(
                "registerSpell",
                RegisterSpellVariables {
                    input: spell.into(),
                    owner: owner.cloned(),
                    lifetime: Some(lifetime),
                },
            )
            .await?;
        fragment.into_result()
    }

    /// Starts a spell training session (`trainSpell`) - the next
    /// completed gesture episodes become the trained spell's
    /// template strokes and corridor; the finished spell registers
    /// persistently (SPELLS.md's learn-by-repetition).
    pub async fn train_spell(
        &self,
        name: &SpellName,
        arming: Option<&SpellArming>,
    ) -> Result<bool, ClientError> {
        self.call_mutation::<TrainSpell, _, _>(
            "trainSpell",
            TrainSpellVariables {
                name: name.clone(),
                arming: arming.map(SpellArmingInput::from),
            },
        )
        .await
    }

    /// Removes a spell (`unregisterSpell`).
    pub async fn unregister_spell(&self, name: &SpellName) -> Result<bool, ClientError> {
        self.call_mutation::<UnregisterSpell, _, _>(
            "unregisterSpell",
            UnregisterSpellVariables { name: name.clone() },
        )
        .await
    }
}
