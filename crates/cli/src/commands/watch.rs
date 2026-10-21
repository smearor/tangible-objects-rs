//! `watch` - live event stream; `LAGGED` surfaces as a warning line.

use futures_lite::StreamExt;
use tangible_objects_client::ClientError;
use tangible_objects_client::EventStreamQuery;
use tangible_objects_client::GraphQLClient;
use tangible_objects_model::PatternId;
use tangible_objects_model::StreamEvent;
use tangible_objects_model::TangibleEvent;
use tangible_objects_model::TangibleSetId;

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
        let mut stream = client.field_events(fields.as_deref()).await?;
        while let Some(event) = stream.next().await {
            print_event(cli.json, event?);
        }
        return Ok(());
    }
    if args.tangibles || args.kinds.is_none() {
        let mut stream = client
            .tangible_events(args.pattern_id.map(PatternId::new))
            .await?;
        while let Some(event) = stream.next().await {
            print_event(cli.json, event?);
        }
        return Ok(());
    }
    let kinds = parse_kinds(args.kinds.as_deref().unwrap_or_default())?;
    let query = EventStreamQuery {
        kinds,
        pattern_id: args.pattern_id.map(PatternId::new),
        set_id: args.set_id.map(TangibleSetId::new),
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

/// Parses a comma-separated `StreamEvent` list (`SCREAMING_SNAKE`
/// wire names).
fn parse_kinds(text: &str) -> Result<Vec<StreamEvent>, ClientError> {
    text.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            serde_json::from_value::<StreamEvent>(serde_json::json!(s.to_uppercase()))
                .map_err(|e| ClientError::Protocol(format!("unknown kind {s}: {e}")))
        })
        .collect()
}
