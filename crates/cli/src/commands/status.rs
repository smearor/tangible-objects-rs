//! `status` - session overview: device, recording, metrics summary.

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;

use crate::args::Cli;

/// `status` - device + recording overview.
pub(crate) async fn status(client: &GraphQLClient, cli: &Cli) -> Result<(), ClientError> {
    let device = client.device_info().await?;
    let recording = client.recording_status().await?;
    if cli.json {
        let v = serde_json::json!({ "deviceInfo": device, "recordingStatus": recording });
        println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
    } else {
        println!(
            "device: {}",
            device
                .and_then(|d| d.name)
                .unwrap_or_else(|| "unknown".into())
        );
        match recording {
            Some(r) if r.active => println!(
                "recording: active label={:?} frames={}",
                r.label.map(|l| l.as_str().to_string()),
                r.recorded_frames.as_u64()
            ),
            _ => println!("recording: inactive"),
        }
    }
    Ok(())
}
