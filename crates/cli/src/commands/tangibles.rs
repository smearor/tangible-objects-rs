//! `tangibles` - currently tracked figures (live poses).

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;

use crate::args::Cli;
use crate::out;

/// `tangibles` - live pose table.
pub(crate) async fn tangibles(client: &GraphQLClient, cli: &Cli) -> Result<(), ClientError> {
    let list = client.tangibles().await?;
    out::emit_table(
        cli.json,
        &list,
        &[
            "ID",
            "Pattern",
            "X (mm)",
            "Y (mm)",
            "Rotation (deg)",
            "Pending",
        ],
        |t| {
            vec![
                t.id.as_u32().to_string(),
                t.pattern_name.as_str().to_string(),
                format!("{:.1}", t.center.x),
                format!("{:.1}", t.center.y),
                format!("{:.1}", t.rotation_rad.to_degrees()),
                t.pending.to_string(),
            ]
        },
    );
    Ok(())
}
