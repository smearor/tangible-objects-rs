//! Subcommand dispatch - every `Command` variant maps 1:1 onto a
//! [`GraphQLClient`] method; one file per topic. No GraphQL strings
//! live here (they belong to `crates/client`).

mod analyze;
mod completion;
mod devices;
mod field_sets;
mod filter;
mod message;
mod metrics;
mod profiles;
mod record;
mod stages;
mod status;
mod tangibles;
mod touches;
mod watch;

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;

use crate::args::Cli;
use crate::args::Command;

/// Dispatches the parsed command onto the client.
pub async fn run(client: &GraphQLClient, cli: &Cli) -> Result<(), ClientError> {
    match &cli.command {
        Command::Devices => devices::devices(client, cli).await?,
        Command::Status => status::status(client, cli).await?,
        Command::Metrics(args) => metrics::metrics(client, args).await?,
        Command::Message(args) => message::message(client, args).await?,
        Command::Tangibles => tangibles::tangibles(client, cli).await?,
        Command::Touches => touches::touches(client, cli).await?,
        Command::Profiles(args) => profiles::profiles(client, cli, args).await?,
        Command::FieldSets(args) => field_sets::field_sets(client, args).await?,
        Command::Stages(args) => stages::stages(client, args).await?,
        Command::Filter(args) => filter::filter(client, args).await?,
        Command::Completion(args) => completion::completion(client, cli, args).await?,
        Command::Record(args) => record::record(client, cli, args).await?,
        Command::Analyze(args) => analyze::analyze(client, args).await?,
        Command::Watch(args) => watch::watch(client, cli, args).await?,
    }
    Ok(())
}
