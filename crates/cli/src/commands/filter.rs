//! `filter` - suppression policy switching (stage-scoped or
//! broadcast).

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;

use crate::args::FilterArgs;

/// `filter <mode> [--stability] [--stage]` - `on` maps to `guarded`
/// (see [`crate::args`]).
pub(crate) async fn filter(client: &GraphQLClient, args: &FilterArgs) -> Result<(), ClientError> {
    client
        .set_filter_mode(args.mode, args.stability, args.stage.clone())
        .await?;
    Ok(())
}
