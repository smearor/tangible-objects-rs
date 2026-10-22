//! `completion` - read or patch the missing-pin completion config.

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;

use crate::args::Cli;
use crate::args::CompletionArgs;
use crate::out;

/// `completion` - no flags reads the config; any flag applies a patch:
/// the current config is read first, merged, then applied.
pub(crate) async fn completion(
    client: &GraphQLClient,
    cli: &Cli,
    args: &CompletionArgs,
) -> Result<(), ClientError> {
    let read_only = args.mode.is_none()
        && args.radius.is_none()
        && args.max_missing.is_none()
        && args.persist.is_none();
    if read_only {
        let config = client.completion().await?;
        out::emit(cli.json, &config, |v| {
            println!("{}", serde_json::to_string_pretty(v).unwrap_or_default())
        });
        return Ok(());
    }
    let mut config = client.completion().await?.unwrap_or_default();
    if let Some(radius) = args.radius {
        config.radius_factor = radius;
    }
    if let Some(max) = args.max_missing {
        config.max_missing = max.max(0) as u32;
    }
    if let Some(persist) = args.persist {
        config.persist = persist;
    }
    if let Some(mode) = args.mode {
        config.mode = mode;
    }
    client.set_completion(&config).await?;
    Ok(())
}
