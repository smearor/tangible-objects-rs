//! `watch` - live event stream; `LAGGED` surfaces as a warning line.

use futures_lite::StreamExt;
use tangible_objects_client::ClientError;
use tangible_objects_client::EventStreamQuery;
use tangible_objects_client::GraphQLClient;
use tangible_objects_model::TangibleEvent;

use crate::args::Cli;
use crate::args::WatchArgs;
use crate::out;

/// `watch` - prints one event line per received event.
pub(crate) async fn watch(
    client: &GraphQLClient,
    cli: &Cli,
    args: &WatchArgs,
) -> Result<(), ClientError> {
    if let Some(fields) = &args.fields {
        let mut stream = client.field_events(fields.as_ref()).await?;
        while let Some(event) = stream.next().await {
            print_event(cli.json, event?);
        }
        return Ok(());
    }
    if args.tangibles || args.kinds.is_empty() {
        let mut stream = client.tangible_events(args.pattern_id).await?;
        while let Some(event) = stream.next().await {
            print_event(cli.json, event?);
        }
        return Ok(());
    }
    let query = EventStreamQuery {
        kinds: args.kinds.clone(),
        pattern_id: args.pattern_id,
        set_id: args.set_id,
        max_hz: args.max_hz,
    };
    let mut stream = client.event_stream(&query).await?;
    while let Some(event) = stream.next().await {
        print_event(cli.json, event?);
    }
    Ok(())
}

/// Prints one subscription event - JSON or compact line.
fn print_event(json: bool, event: TangibleEvent) {
    if json {
        println!("{}", serde_json::to_string(&event).unwrap_or_default());
    } else {
        println!("{}", out::event_line(&event));
    }
}
