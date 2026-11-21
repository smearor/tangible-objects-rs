//! `spells` - multi-finger gesture registration, removal and
//! remote training.

use std::path::Path;

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;
use tangible_objects_model::RegisterSpellResult;
use tangible_objects_model::Spell;

use crate::args::Cli;
use crate::args::SpellsArgs;
use crate::args::SpellsCommand;

/// `spells` subcommands.
pub(crate) async fn spells(
    client: &GraphQLClient,
    _cli: &Cli,
    args: &SpellsArgs,
) -> Result<(), ClientError> {
    match &args.command {
        SpellsCommand::Register {
            file,
            owner,
            lifetime,
        } => {
            let spell = load_spell(file)?;
            match client
                .register_spell(&spell, owner.as_ref(), *lifetime)
                .await?
            {
                RegisterSpellResult::Registered(registered) => {
                    println!("registered: {}", registered.name);
                }
                RegisterSpellResult::Rejected(rejected) => {
                    return Err(ClientError::Protocol(rejected.reason));
                }
            }
        }
        SpellsCommand::Unregister { name } => {
            client.unregister_spell(name).await?;
        }
        SpellsCommand::Train { name, arming } => {
            client.train_spell(name, arming.as_ref()).await?;
        }
    }
    Ok(())
}

/// Loads a spell `.toml` for `spells register`.
fn load_spell(file: &Path) -> Result<Spell, ClientError> {
    Spell::load(file)
        .map_err(|e| ClientError::Protocol(format!("spell load {}: {e}", file.display())))
}
