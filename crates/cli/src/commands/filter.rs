//! `filter` - suppression policy switching (stage-scoped or
//! broadcast).

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;
use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::StageName;

use crate::args::FilterArgs;

/// `filter <mode> [--stability] [--stage]` - `on` maps to `guarded`,
/// `off` to `off`.
pub(crate) async fn filter(client: &GraphQLClient, args: &FilterArgs) -> Result<(), ClientError> {
    let mode = parse_filter_mode(&args.mode)?;
    let stability = args.stability.map(FrameCount::new);
    client
        .set_filter_mode(mode, stability, args.stage.as_deref().map(StageName::new))
        .await?;
    Ok(())
}

/// Parses `filter on|off|<mode>` (`on` maps to `guarded`).
fn parse_filter_mode(text: &str) -> Result<FilterMode, ClientError> {
    let normalized = match text {
        "on" => "guarded",
        other => other,
    };
    normalized
        .parse::<FilterMode>()
        .map_err(|()| ClientError::Protocol(format!("unknown filter mode {text}")))
}
