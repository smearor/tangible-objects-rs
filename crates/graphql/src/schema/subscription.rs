use async_graphql::Context;
use async_graphql::Result;
use async_graphql::Subscription;
use futures_core::Stream;
use tangible_objects_model::EventCount;
use tangible_objects_model::FieldSetName;
use tangible_objects_model::PatternId;
use tangible_objects_model::TangibleSetId;
use tangible_objects_runtime::EngineHandle;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

use crate::StreamEvent;
use crate::TangibleEvent;
use crate::api::EventFilter;

/// Live domain event stream.
///
/// Streams are deltas only - a client that needs initial state runs the
/// `snapshot`/`profiles` queries first, then subscribes (SUBSCRIPTIONS.md:
/// no snapshot push on connect). Broadcast lag surfaces as a `LAGGED`
/// control event carrying `skipped`, never as a silent gap or failure.
pub struct SubscriptionRoot;

impl SubscriptionRoot {
    /// Shared stream setup: subscribe to the broadcast channel, translate
    /// lag into `LAGGED` events, then run the server-side filter
    /// (kind + ids + `TOUCH_POINTS_UPDATED` throttle).
    fn stream(handle: &EngineHandle, filter: EventFilter) -> impl Stream<Item = TangibleEvent> {
        let mut filter = filter;
        BroadcastStream::new(handle.subscribe())
            .filter_map(|result| match result {
                Ok(event) => Some(TangibleEvent::from(&event)),
                Err(BroadcastStreamRecvError::Lagged(skipped)) => {
                    Some(TangibleEvent::lagged(EventCount::from(skipped)))
                }
            })
            .filter(move |event| filter.accepts(event))
    }

    /// Validates the `kinds` filter: an empty list would silently mean
    /// "everything", which the schema rejects instead.
    fn check_kinds(kinds: &[StreamEvent]) -> Result<()> {
        if kinds.is_empty() {
            return Err(async_graphql::Error::new(
                "kinds must not be empty - request at least one StreamEvent kind",
            ));
        }
        Ok(())
    }
}

#[Subscription]
impl SubscriptionRoot {
    /// The single subscription entry point: streams the requested
    /// `StreamEvent` kinds, narrowed by `patternId`/`setId`; `maxHz`
    /// throttles the high-rate `TOUCH_POINTS_UPDATED` kind (default 30).
    async fn event_stream(
        &self,
        ctx: &Context<'_>,
        kinds: Vec<StreamEvent>,
        pattern_id: Option<PatternId>,
        set_id: Option<TangibleSetId>,
        max_hz: Option<u32>,
    ) -> Result<impl Stream<Item = TangibleEvent>> {
        Self::check_kinds(&kinds)?;
        let handle = ctx.data::<EngineHandle>()?;
        Ok(Self::stream(
            handle,
            EventFilter::new(&kinds, pattern_id, set_id, None, max_hz),
        ))
    }

    /// Tangible lifecycle + motion stream - `kinds` preset to
    /// `TANGIBLE_ADDED`/`TANGIBLE_MOVED`/`TANGIBLE_REMOVED`; `patternId`
    /// narrows to one figure.
    async fn tangible_events(
        &self,
        ctx: &Context<'_>,
        pattern_id: Option<PatternId>,
    ) -> Result<impl Stream<Item = TangibleEvent>> {
        let handle = ctx.data::<EngineHandle>()?;
        Ok(Self::stream(
            handle,
            EventFilter::new(
                &[
                    StreamEvent::TangibleAdded,
                    StreamEvent::TangibleMoved,
                    StreamEvent::TangibleRemoved,
                ],
                pattern_id,
                None,
                None,
                None,
            ),
        ))
    }

    /// Occupancy field stream: `FIELD_PLACED`/`FIELD_LIFTED`/
    /// `FIELD_COUNT_CHANGED`, narrowed to one field set when `set` names it.
    async fn field_events(
        &self,
        ctx: &Context<'_>,
        set: Option<FieldSetName>,
    ) -> Result<impl Stream<Item = TangibleEvent>> {
        let handle = ctx.data::<EngineHandle>()?;
        Ok(Self::stream(
            handle,
            EventFilter::new(
                &[
                    StreamEvent::FieldPlaced,
                    StreamEvent::FieldLifted,
                    StreamEvent::FieldCountChanged,
                ],
                None,
                None,
                set,
                None,
            ),
        ))
    }

    /// Streams every domain event unfiltered.
    #[graphql(deprecation = "use eventStream(kinds: [...]) - an explicit filter documents intent")]
    async fn events(&self, ctx: &Context<'_>) -> Result<impl Stream<Item = TangibleEvent>> {
        let handle = ctx.data::<EngineHandle>()?;
        Ok(Self::stream(
            handle,
            EventFilter::new(
                &[
                    StreamEvent::TangibleAdded,
                    StreamEvent::TangibleMoved,
                    StreamEvent::TangibleRemoved,
                    StreamEvent::TouchPointsUpdated,
                    StreamEvent::FieldPlaced,
                    StreamEvent::FieldLifted,
                    StreamEvent::FieldCountChanged,
                    StreamEvent::SetAdded,
                    StreamEvent::SetRemoved,
                    StreamEvent::ObjectAdded,
                    StreamEvent::ObjectRemoved,
                    StreamEvent::ObjectMoved,
                    StreamEvent::ProfileRegistered,
                    StreamEvent::ProfileUnregistered,
                    StreamEvent::FieldSetRegistered,
                    StreamEvent::FieldSetUnregistered,
                    StreamEvent::CompletionChanged,
                    StreamEvent::FilterModeChanged,
                    StreamEvent::RecordingStarted,
                    StreamEvent::RecordingStopped,
                    StreamEvent::ConfigReloaded,
                ],
                None,
                None,
                None,
                None,
            ),
        ))
    }
}
