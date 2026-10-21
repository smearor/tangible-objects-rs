//! `subscribe()` - connects and runs one `graphql-transport-ws`
//! subscription through `graphql-ws-client`.

use futures_lite::Stream;
use futures_lite::stream::StreamExt;
use serde::Serialize;
use tangible_objects_model::TangibleEvent;

use crate::ClientError;
use crate::Endpoint;
use crate::ws::connection::TungsteniteConnection;
use crate::ws::operation::EventOperation;

/// Connects and sends `payload` as a `subscribe` operation, then
/// yields decoded [`TangibleEvent`]s until `complete`/`error` or
/// transport end.
pub(crate) async fn subscribe(
    endpoint: &Endpoint,
    field: &'static str,
    payload: impl Serialize,
) -> Result<impl Stream<Item = Result<TangibleEvent, ClientError>> + 'static, ClientError> {
    let payload = serde_json::to_value(payload).map_err(ClientError::Decode)?;
    let connection = TungsteniteConnection::connect(endpoint).await?;
    let subscription = graphql_ws_client::Client::build(connection)
        .subscribe(EventOperation::new(payload, field))
        .await?;
    Ok(subscription.map(|item| item.map_err(ClientError::from)))
}
