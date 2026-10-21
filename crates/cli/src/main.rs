//! `tangible-objects-cli` - remote CLI client for the daemon's
//! GraphQL endpoint.

mod args;
mod commands;
mod out;

use clap::Parser;
use miette::Context;
use miette::IntoDiagnostic;
use miette::Result;
use tangible_objects_client::Endpoint;
use tangible_objects_client::GraphQLClient;

#[tokio::main]
async fn main() -> Result<()> {
    let args = args::Cli::parse();
    let endpoint = Endpoint::resolve(args.endpoint.clone());
    let client = GraphQLClient::new(endpoint)
        .into_diagnostic()
        .wrap_err("endpoint invalid")?;
    commands::run(&client, &args).await.into_diagnostic()
}
