//! `ServerHandler` impl for [`EngineMcp`] - server info, resource
//! listing/reading, `subscriptions/listen` support, and the merged
//! tool router collecting every topic module's named router.

use rmcp::ErrorData;
use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::ListResourcesResult;
use rmcp::model::PaginatedRequestParams;
use rmcp::model::ReadResourceRequestParams;
use rmcp::model::ReadResourceResponse;
use rmcp::model::ReadResourceResult;
use rmcp::model::ResourceContents;
use rmcp::model::ServerCapabilities;
use rmcp::model::ServerConfig;
use rmcp::model::SubscriptionFilter;
use rmcp::service::RequestContext;
use rmcp::service::RoleServer;
use rmcp::service::SubscriptionContext;
use rmcp::tool_handler;
use tangible_objects_model::DomainEvent;
use tangible_objects_model::EngineEvent;
use tangible_objects_model::LifecycleEvent;
use tangible_objects_model::TangibleSetState;
use tokio::sync::broadcast::error::RecvError;
use tracing::debug;
use tracing::warn;

use crate::server::engine_mcp::EngineMcp;
use crate::server::helpers::json_pretty;
use crate::server::helpers::resource;

/// Resource URI listing the currently tracked tangibles.
const URI_TANGIBLES: &str = "tangible://tangibles";
/// Resource URI listing the active touch points.
const URI_TOUCH_POINTS: &str = "tangible://touch-points";
/// Resource URI listing the configured sets with activation state.
const URI_SETS: &str = "tangible://sets";
/// Resource URI listing the loaded figure profiles.
const URI_PROFILES: &str = "tangible://profiles";
/// Resource URI listing the registered occupancy field sets.
const URI_FIELD_SETS: &str = "tangible://field-sets";
/// Resource URI for the live recording status (`null` while nothing
/// records).
const URI_RECORDING: &str = "tangible://recording";
/// Resource URI for the user-facing message currently shown in the
/// GUI overlay (`""` when none is set).
const URI_MESSAGE: &str = "tangible://message";

/// Resource URIs that emit `notifications/resources/updated` - every
/// resource backed by a [`DomainEvent`]. `URI_MESSAGE` is excluded:
/// `ShowMessage` broadcasts no event, so a subscription would be
/// silent; the resource stays readable.
const SUBSCRIBABLE_URIS: [&str; 6] = [
    URI_TANGIBLES,
    URI_TOUCH_POINTS,
    URI_SETS,
    URI_PROFILES,
    URI_FIELD_SETS,
    URI_RECORDING,
];

/// Maps one domain event onto the resource URIs it invalidates -
/// drives `notifications/resources/updated` inside `listen`.
fn resource_uris(event: &DomainEvent) -> &'static [&'static str] {
    match event {
        DomainEvent::Engine(
            EngineEvent::TangibleAdded(_)
            | EngineEvent::TangibleMoved(_)
            | EngineEvent::TangibleRemoved(_),
        ) => &[URI_TANGIBLES],
        DomainEvent::Engine(EngineEvent::TouchPointsUpdated) => &[URI_TOUCH_POINTS],
        DomainEvent::Engine(
            EngineEvent::FieldPlaced { .. }
            | EngineEvent::FieldLifted { .. }
            | EngineEvent::FieldCountChanged { .. },
        ) => &[URI_FIELD_SETS],
        DomainEvent::Set(_) => &[URI_SETS],
        DomainEvent::Lifecycle(
            LifecycleEvent::ProfileRegistered { .. } | LifecycleEvent::ProfileUnregistered { .. },
        ) => &[URI_PROFILES],
        DomainEvent::Lifecycle(
            LifecycleEvent::FieldSetRegistered { .. } | LifecycleEvent::FieldSetUnregistered { .. },
        ) => &[URI_FIELD_SETS],
        DomainEvent::Lifecycle(
            LifecycleEvent::RecordingStarted | LifecycleEvent::RecordingStopped { .. },
        ) => &[URI_RECORDING],
        // "Refresh your view" marker - invalidates every resource.
        DomainEvent::Lifecycle(LifecycleEvent::ConfigReloaded) => &SUBSCRIBABLE_URIS,
        // Stage/completion/filter-mode lifecycle has no resource
        // counterpart - nothing to invalidate.
        DomainEvent::Lifecycle(_) => &[],
    }
}

impl EngineMcp {
    /// Merges every topic router into the server-wide tool router.
    fn tool_router() -> ToolRouter<Self> {
        let mut router = Self::analysis_router();
        router.merge(Self::completion_router());
        router.merge(Self::device_router());
        router.merge(Self::field_set_router());
        router.merge(Self::filter_router());
        router.merge(Self::gui_router());
        router.merge(Self::metrics_router());
        router.merge(Self::message_router());
        router.merge(Self::owner_router());
        router.merge(Self::pattern_router());
        router.merge(Self::profiles_router());
        router.merge(Self::recording_router());
        router.merge(Self::stage_router());
        router.merge(Self::tangibles_router());
        router.merge(Self::training_router());
        router
    }
}

#[tool_handler]
impl ServerHandler for EngineMcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_resources_subscribe()
                .build(),
        )
        .with_instructions(
            "Tangible object detection engine on a multitouch surface. \
             Tools expose the live detection state and control commands; \
             resources under tangible:// provide read-only snapshots; \
             subscriptions/listen pushes notifications/resources/updated \
             for the event-backed URIs.",
        )
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, ErrorData>> + Send + '_ {
        std::future::ready(Ok(ListResourcesResult::with_all_items(vec![
            resource(URI_TANGIBLES, "tangibles", "Currently tracked tangibles"),
            resource(URI_TOUCH_POINTS, "touch-points", "Active touch points"),
            resource(
                URI_SETS,
                "sets",
                "Tangible Object Sets with activation state",
            ),
            resource(URI_PROFILES, "profiles", "Loaded figure profiles"),
            resource(
                URI_FIELD_SETS,
                "field-sets",
                "Registered occupancy field sets",
            ),
            resource(URI_RECORDING, "recording", "Live recording status"),
            resource(
                URI_MESSAGE,
                "message",
                "User-facing message currently shown in the GUI overlay",
            ),
        ])))
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceResponse, ErrorData>> + Send + '_ {
        let handle = self.handle.clone();
        async move {
            let body = match request.uri.as_str() {
                URI_TANGIBLES => json_pretty(&handle.snapshot().tracked)?,
                URI_TOUCH_POINTS => json_pretty(&handle.snapshot().touches)?,
                URI_SETS => {
                    let active = handle.snapshot().active_sets;
                    json_pretty(
                        &handle
                            .sets()
                            .iter()
                            .map(|set| TangibleSetState::from((set, active.contains(&set.id))))
                            .collect::<Vec<_>>(),
                    )?
                }
                URI_PROFILES => json_pretty(&handle.profiles())?,
                URI_FIELD_SETS => json_pretty(&handle.snapshot().field_sets)?,
                URI_RECORDING => json_pretty(&handle.snapshot().recording)?,
                URI_MESSAGE => handle.snapshot().message.unwrap_or_default(),
                uri => {
                    return Err(ErrorData::resource_not_found(
                        format!("unknown resource: {uri}"),
                        None,
                    ));
                }
            };
            Ok(ReadResourceResult::new(vec![
                ResourceContents::text(body, request.uri).with_mime_type("application/json"),
            ])
            .into())
        }
    }

    /// Accepts the requested URIs that map onto a domain event -
    /// the SDK intersects this with the requested filter and the
    /// advertised `resources.subscribe` capability.
    fn accepted_subscription_filter(
        &self,
        requested: &SubscriptionFilter,
    ) -> Option<SubscriptionFilter> {
        let uris = requested
            .resource_subscriptions
            .iter()
            .flatten()
            .filter(|uri| SUBSCRIBABLE_URIS.contains(&uri.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        Some(
            SubscriptionFilter::builder()
                .resource_subscriptions(uris)
                .build(),
        )
    }

    /// Forwards domain events of the accepted URIs as
    /// `notifications/resources/updated` until the subscription is
    /// cancelled or the event stream closes.
    fn listen(
        &self,
        context: SubscriptionContext,
    ) -> impl Future<Output = Result<(), ErrorData>> + Send + '_ {
        let handle = self.handle.clone();
        async move {
            let sink = context.sink().clone();
            let mut events = handle.subscribe();
            loop {
                tokio::select! {
                    biased;
                    _ = context.cancelled() => break,
                    event = events.recv() => match event {
                        Ok(event) => {
                            for uri in resource_uris(&event) {
                                if let Err(err) = sink.notify_resource_updated(*uri).await {
                                    debug!(%err, %uri, "resource update notification failed");
                                }
                            }
                        }
                        Err(RecvError::Lagged(skipped)) => {
                            warn!(skipped, "MCP subscription lagged");
                        }
                        Err(RecvError::Closed) => break,
                    }
                }
            }
            Ok(())
        }
    }
}
