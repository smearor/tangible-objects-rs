//! `field-sets` - occupancy field-set registration.

use std::path::Path;

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;
use tangible_objects_model::FieldSet;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;

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
            let set = FieldSet::load(Path::new(file))
                .map_err(|e| ClientError::Protocol(format!("field-set load: {e}")))?;
            let owner = owner.as_deref().map(Owner::new);
            let lifetime = lifetime
                .as_deref()
                .map(|l| {
                    serde_json::from_value::<Lifetime>(serde_json::json!(l))
                        .map_err(|e| ClientError::Protocol(format!("unknown lifetime {l}: {e}")))
                })
                .transpose()?
                .unwrap_or_default();
            client
                .register_field_set(&set, owner.as_ref(), lifetime)
                .await?;
        }
        FieldSetsCommand::Unregister { name } => {
            client.unregister_field_set(name).await?;
        }
    }
    Ok(())
}
