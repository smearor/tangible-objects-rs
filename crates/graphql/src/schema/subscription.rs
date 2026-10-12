use async_graphql::Context;
use async_graphql::Result;
use async_graphql::Subscription;
use futures_core::Stream;
use tangible_objects_runtime::EngineHandle;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::TangibleEvent;

/// Live domain event stream.
pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    /// Streams domain events: tangible added/moved/removed, touch point
    /// updates and Tangible Object Set actions.
    ///
    /// Lagging subscribers drop the oldest events; the stream ends when the
    /// device loop stops.
    async fn events(&self, ctx: &Context<'_>) -> Result<impl Stream<Item = TangibleEvent>> {
        let handle = ctx.data::<EngineHandle>()?;
        let stream = BroadcastStream::new(handle.subscribe())
            .filter_map(|result| result.ok())
            .map(|event| TangibleEvent::from(&event));
        Ok(stream)
    }
}
