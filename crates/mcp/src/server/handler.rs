//! `ServerHandler` impl for [`EngineMcp`] - server info, resource
//! listing/reading, and the merged tool router collecting every
//! topic module's named router.

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
use rmcp::service::RequestContext;
use rmcp::service::RoleServer;
use rmcp::tool_handler;
use tangible_objects_model::TangibleSetState;

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
                .build(),
        )
        .with_instructions(
            "Tangible object detection engine on a multitouch surface. \
             Tools expose the live detection state and control commands; \
             resources under tangible:// provide read-only snapshots.",
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
}
