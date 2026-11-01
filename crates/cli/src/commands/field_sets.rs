//! `field-sets` - occupancy field-set registration.

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;
use tangible_objects_model::FieldSet;
use tangible_objects_model::RegisterFieldSetResult;

use crate::args::Cli;
use crate::args::FieldSetsArgs;
use crate::args::FieldSetsCommand;

/// `field-sets` subcommands.
pub(crate) async fn field_sets(
    client: &GraphQLClient,
    cli: &Cli,
    args: &FieldSetsArgs,
) -> Result<(), ClientError> {
    match &args.command {
        FieldSetsCommand::List => {
            let list = client.field_sets().await?;
            crate::out::emit_table(
                cli.json,
                &list,
                &["Name", "Fields", "Stability", "Window", "Suppress"],
                |set| {
                    vec![
                        set.name.as_str().to_string(),
                        set.fields.len().to_string(),
                        set.stability.to_string(),
                        set.window
                            .as_ref()
                            .map(|w| match (w.width, w.height) {
                                (Some(width), Some(height)) => {
                                    format!("{},{} {}×{}", w.x_offset, w.y_offset, width, height)
                                }
                                _ => format!("{},{}", w.x_offset, w.y_offset),
                            })
                            .unwrap_or_default(),
                        set.suppress.to_string(),
                    ]
                },
            );
        }
        FieldSetsCommand::Register {
            file,
            owner,
            lifetime,
        } => {
            let set = FieldSet::load(file)
                .map_err(|e| ClientError::Protocol(format!("field-set load: {e}")))?;
            match client
                .register_field_set(&set, owner.as_ref(), lifetime.unwrap_or_default())
                .await?
            {
                RegisterFieldSetResult::Registered(registered) => {
                    println!("registered: {}", registered.name);
                }
                RegisterFieldSetResult::Rejected(rejected) => {
                    if let Some(overlap) = &rejected.overlap {
                        println!(
                            "rejected: '{}' overlaps '{}' in region {}",
                            overlap.first, overlap.second, overlap.region
                        );
                    }
                    return Err(ClientError::Protocol(rejected.reason));
                }
            }
        }
        FieldSetsCommand::Unregister { name } => {
            client.unregister_field_set(name).await?;
        }
    }
    Ok(())
}
