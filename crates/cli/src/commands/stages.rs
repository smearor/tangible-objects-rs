//! `stages` - pipeline stage lifecycle.

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;
use tangible_objects_model::StageName;

use crate::args::StagesArgs;
use crate::args::StagesCommand;

/// `stages` subcommands.
pub(crate) async fn stages(client: &GraphQLClient, args: &StagesArgs) -> Result<(), ClientError> {
    match &args.command {
        StagesCommand::Register { name } => {
            client.register_stage(&StageName::new(name)).await?;
        }
        StagesCommand::Unregister { name } => {
            client.unregister_stage(&StageName::new(name)).await?;
        }
    }
    Ok(())
}
