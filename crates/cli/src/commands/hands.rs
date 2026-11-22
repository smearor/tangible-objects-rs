//! `hands` - handprint registration and removal (SPELLS.md's
//! per-user spells).

use std::path::Path;

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;
use tangible_objects_model::HandProfile;
use tangible_objects_model::RegisterHandProfileResult;

use crate::args::Cli;
use crate::args::HandsArgs;
use crate::args::HandsCommand;

/// `hands` subcommands.
pub(crate) async fn hands(
    client: &GraphQLClient,
    _cli: &Cli,
    args: &HandsArgs,
) -> Result<(), ClientError> {
    match &args.command {
        HandsCommand::Register {
            file,
            owner,
            lifetime,
        } => {
            let hand = load_hand(file)?;
            match client
                .register_hand_profile(&hand, owner.as_ref(), *lifetime)
                .await?
            {
                RegisterHandProfileResult::Registered(registered) => {
                    println!("registered: {}", registered.name);
                }
                RegisterHandProfileResult::Rejected(rejected) => {
                    return Err(ClientError::Protocol(rejected.reason));
                }
            }
        }
        HandsCommand::Unregister { name } => {
            client.unregister_hand_profile(name).await?;
        }
    }
    Ok(())
}

/// Loads a hand profile `.toml` for `hands register`.
fn load_hand(file: &Path) -> Result<HandProfile, ClientError> {
    HandProfile::load(file)
        .map_err(|e| ClientError::Protocol(format!("hand profile load {}: {e}", file.display())))
}
