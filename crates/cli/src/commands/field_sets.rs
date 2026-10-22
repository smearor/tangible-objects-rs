//! `field-sets` - occupancy field-set registration.

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;
use tangible_objects_model::FieldSet;

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
            client
                .register_field_set(&set, owner.as_ref(), lifetime.unwrap_or_default())
                .await?;
        }
        FieldSetsCommand::Unregister { name } => {
            client.unregister_field_set(name).await?;
        }
    }
    Ok(())
}
