//! `profiles` - figure profile listing, registration, updating and
//! remote training.

use std::path::Path;

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;
use tangible_objects_client::ProfileQuery;
use tangible_objects_model::ProfileFile;

use crate::args::Cli;
use crate::args::ProfilesArgs;
use crate::args::ProfilesCommand;
use crate::out;

/// `profiles` subcommands.
pub(crate) async fn profiles(
    client: &GraphQLClient,
    cli: &Cli,
    args: &ProfilesArgs,
) -> Result<(), ClientError> {
    match &args.command {
        ProfilesCommand::List { owner, static_only } => {
            let query = ProfileQuery {
                owner: owner.clone(),
                static_only: static_only.then_some(true),
            };
            let list = client.profiles(&query).await?;
            out::emit_table(
                cli.json,
                &list,
                &["ID", "Name", "Pins", "Owner", "Stage"],
                |p| {
                    vec![
                        p.pattern_id.as_u32().to_string(),
                        p.name.as_str().to_string(),
                        p.num_points.to_string(),
                        p.owner
                            .as_ref()
                            .map(|o| o.as_str().to_string())
                            .unwrap_or_default(),
                        p.stage
                            .as_ref()
                            .map(|s| s.as_str().to_string())
                            .unwrap_or_default(),
                    ]
                },
            );
        }
        ProfilesCommand::Show { id } => {
            let list = client.profiles(&ProfileQuery::default()).await?;
            let found = list.iter().find(|p| p.pattern_id == *id);
            match found {
                Some(p) => println!("{}", serde_json::to_string_pretty(p).unwrap_or_default()),
                None => eprintln!("no profile with id {id}"),
            }
        }
        ProfilesCommand::Delete { id } => {
            client.unregister_pattern(*id).await?;
        }
        ProfilesCommand::UnregisterOwner { owner } => {
            client.unregister_owner(owner).await?;
        }
        ProfilesCommand::Register {
            file,
            owner,
            lifetime,
        } => {
            let profile = load_profile(file)?;
            client
                .register_pattern(&profile, owner.as_ref(), *lifetime)
                .await?;
        }
        ProfilesCommand::Update { id, file } => {
            let profile = load_profile(file)?;
            client.update_profile(*id, &profile).await?;
        }
        ProfilesCommand::Train { name } => {
            client.train_profile(name).await?;
        }
    }
    Ok(())
}

/// Loads a profile `.toml` for `profiles register|update`.
fn load_profile(file: &Path) -> Result<ProfileFile, ClientError> {
    ProfileFile::load(file)
        .map_err(|e| ClientError::Protocol(format!("profile load {}: {e}", file.display())))
}
