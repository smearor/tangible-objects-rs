//! `field-sets` - occupancy field-set registration.

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;
use tangible_objects_model::FieldSet;
use tangible_objects_model::RegisterFieldSetResult;

use crate::args::FieldSetsArgs;
use crate::args::FieldSetsCommand;

/// `field-sets` subcommands.
pub(crate) async fn field_sets(
    client: &GraphQLClient,
    args: &FieldSetsArgs,
) -> Result<(), ClientError> {
    match &args.command {
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
