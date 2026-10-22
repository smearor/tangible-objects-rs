//! `analyze` - offline analysis queries; reports print verbatim
//! (JSON strings).

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;

use crate::args::AnalyzeArgs;
use crate::args::AnalyzeCommand;

/// `analyze` subcommands.
pub(crate) async fn analyze(client: &GraphQLClient, args: &AnalyzeArgs) -> Result<(), ClientError> {
    match &args.command {
        AnalyzeCommand::Recording {
            path,
            mode,
            profiles_dir,
        } => {
            let report = client
                .analyze_recording(path, mode.unwrap_or_default(), profiles_dir.as_deref())
                .await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_default()
            );
        }
        AnalyzeCommand::Similarity { profiles_dir } => {
            let report = client.analyze_similarity(profiles_dir.as_deref()).await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_default()
            );
        }
        AnalyzeCommand::Artifact { path } => {
            println!("{}", client.recording_artifact(path).await?);
        }
    }
    Ok(())
}
