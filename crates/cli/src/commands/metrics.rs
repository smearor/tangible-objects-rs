//! `metrics` - session metrics; `metrics reset` clears them.

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;

use crate::args::MetricsArgs;
use crate::args::MetricsCommand;

/// `metrics` / `metrics reset`.
pub(crate) async fn metrics(client: &GraphQLClient, args: &MetricsArgs) -> Result<(), ClientError> {
    match &args.command {
        Some(MetricsCommand::Reset) => {
            client.reset_metrics().await?;
            println!("metrics reset");
        }
        None => {
            let m = client.metrics().await?;
            println!("{}", serde_json::to_string_pretty(&m).unwrap_or_default());
        }
    }
    Ok(())
}
