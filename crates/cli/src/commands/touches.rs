//! `touches` - active touch points.

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;

use crate::args::Cli;
use crate::out;

/// `touches` - active contact table.
pub(crate) async fn touches(client: &GraphQLClient, cli: &Cli) -> Result<(), ClientError> {
    let list = client.touch_points().await?;
    out::emit_table(cli.json, &list, &["Slot", "Tracking ID", "X", "Y"], |t| {
        vec![
            t.slot.as_u32().to_string(),
            t.tracking_id.as_i32().to_string(),
            format!("{:.1}", t.pos.x),
            format!("{:.1}", t.pos.y),
        ]
    });
    Ok(())
}
