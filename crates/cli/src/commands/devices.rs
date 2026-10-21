//! `devices` - input device info (name + axis calibration).

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;

use crate::args::Cli;
use crate::out;

/// `devices` - input device info.
pub(crate) async fn devices(client: &GraphQLClient, cli: &Cli) -> Result<(), ClientError> {
    let info = client.device_info().await?;
    out::emit(cli.json, &info, |v| {
        println!("{}", serde_json::to_string_pretty(v).unwrap_or_default())
    });
    Ok(())
}
