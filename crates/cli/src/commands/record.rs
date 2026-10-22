//! `record` - recording session control.

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;

use crate::args::Cli;
use crate::args::RecordArgs;
use crate::args::RecordCommand;
use crate::out;

/// `record` subcommands.
pub(crate) async fn record(
    client: &GraphQLClient,
    cli: &Cli,
    args: &RecordArgs,
) -> Result<(), ClientError> {
    match &args.command {
        RecordCommand::Status => {
            let status = client.recording_status().await?;
            out::emit(cli.json, &status, |v| {
                println!("{}", serde_json::to_string_pretty(v).unwrap_or_default())
            });
        }
        RecordCommand::Start { path, label } => {
            client.start_recording(path, label.as_ref()).await?;
        }
        RecordCommand::Stop => {
            client.stop_recording().await?;
        }
        RecordCommand::Label { label, clear } => {
            let value = if *clear { None } else { label.as_ref() };
            client.set_record_label(value).await?;
        }
    }
    Ok(())
}
