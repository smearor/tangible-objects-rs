use std::io;
use std::net::SocketAddr;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use async_graphql_axum::GraphQLSubscription;
use axum::Router;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::routing::post_service;
use tangible_objects_runtime::EngineHandle;
use tracing::error;
use tracing::info;

use super::EngineSchema;
use super::MutationRoot;
use super::QueryRoot;
use super::SubscriptionRoot;

/// Builds the engine schema with the runtime handle in its data.
pub fn build_schema(handle: EngineHandle) -> EngineSchema {
    EngineSchema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(handle)
        .finish()
}

/// Serves the GraphQL endpoint.
///
/// Routes: `GET /` serves GraphiQL, `POST /graphql` executes queries and
/// mutations, `GET /graphql` upgrades to WebSocket for subscriptions.
pub async fn serve(handle: EngineHandle, addr: SocketAddr) -> io::Result<()> {
    let schema = build_schema(handle);
    let app = Router::new().route("/", get(graphiql)).route(
        "/graphql",
        post_service(GraphQL::new(schema.clone())).get_service(GraphQLSubscription::new(schema)),
    );
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(%addr, "GraphQL endpoint listening");
    axum::serve(listener, app).await
}

/// Spawns the GraphQL server on a dedicated thread with its own tokio
/// runtime, keeping the daemon's synchronous device loop untouched.
pub fn spawn(handle: EngineHandle, addr: SocketAddr) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let result = tokio::runtime::Runtime::new()
            .and_then(|runtime| runtime.block_on(serve(handle, addr)));
        if let Err(err) = result {
            error!(%err, %addr, "GraphQL server failed");
        }
    })
}

/// GraphiQL IDE for manual queries during development.
async fn graphiql() -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .endpoint("/graphql")
            .subscription_endpoint("/graphql")
            .finish(),
    )
}

#[cfg(test)]
mod tests {
    use super::build_schema;
    use tangible_objects_engine::EngineEvent;
    use tangible_objects_engine::EngineFrame;
    use tangible_objects_model::DEFAULT_SPAWN_STABILITY;
    use tangible_objects_model::FilterMode;
    use tangible_objects_model::FrameCount;
    use tangible_objects_model::PatternId;
    use tangible_objects_model::PinCount;
    use tangible_objects_model::RotationRad;
    use tangible_objects_model::TangibleId;
    use tangible_objects_model::TrackedTangible;
    use tangible_objects_model::Vec2;
    use tangible_objects_runtime::ControlCommand;
    use tangible_objects_runtime::EngineRuntime;
    use tangible_objects_runtime::ProfileInfo;

    fn tracked() -> Vec<TrackedTangible> {
        vec![TrackedTangible {
            id: TangibleId::new(7),
            pattern_id: PatternId::new(1),
            pattern_name: "triangle".into(),
            center: Vec2::new(10.0, 20.0),
            rotation_rad: RotationRad::new(1.0),
            pending: false,
            pending_frames: FrameCount::new(0),
            spawn_stability: DEFAULT_SPAWN_STABILITY,
            frames_since_match: FrameCount::new(0),
        }]
    }

    #[tokio::test]
    async fn tangibles_query_reads_the_snapshot() {
        let (mut runtime, handle) = EngineRuntime::new(
            Vec::new(),
            vec![ProfileInfo {
                pattern_id: PatternId::new(1),
                name: "triangle".into(),
                num_points: PinCount::new(3),
                owner: None,
                spawn_on_partial: false,
                spawn_stability: DEFAULT_SPAWN_STABILITY,
                stage: None,
            }],
        );
        let schema = build_schema(handle);

        runtime.publish(
            &EngineFrame {
                matches: Vec::new(),
                touches: Vec::new(),
                events: vec![EngineEvent::TangibleAdded(tracked()[0].clone())],
                candidates: Vec::new(),
                ran_generic: true,
            },
            (&tracked()).into(),
        );

        let result = schema
            .execute("{ tangibles { id patternId patternName rotationDeg } profiles { name numPoints } }")
            .await;
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

        let data = result.data.into_json().expect("data as json");
        assert_eq!(data["tangibles"][0]["id"], 7);
        assert_eq!(data["tangibles"][0]["patternName"], "triangle");
        assert_eq!(data["profiles"][0]["name"], "triangle");
        assert_eq!(data["profiles"][0]["numPoints"], 3);
    }

    #[tokio::test]
    async fn set_filter_mode_mutation_reaches_the_control_channel() {
        let (mut runtime, handle) = EngineRuntime::new(Vec::new(), Vec::new());
        let schema = build_schema(handle);

        let result = schema
            .execute("mutation { setFilterMode(mode: GUARDED, stability: 3) }")
            .await;
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

        assert!(matches!(
            runtime.try_recv_command(),
            Ok(ControlCommand::SetFilterMode {
                mode: FilterMode::Guarded,
                stability: Some(_),
                stage: None,
            })
        ));
    }

    #[tokio::test]
    async fn event_stream_rejects_an_empty_kinds_filter() {
        use tokio_stream::StreamExt;

        let (_runtime, handle) = EngineRuntime::new(Vec::new(), Vec::new());
        let schema = build_schema(handle);

        let mut stream = schema.execute_stream("subscription { eventStream(kinds: []) { kind } }");
        let response = tokio::time::timeout(std::time::Duration::from_secs(2), stream.next())
            .await
            .expect("response within timeout")
            .expect("a response item");
        assert!(
            response.errors.iter().any(|e| e.message.contains("kinds")),
            "expected kinds validation error, got: {:?}",
            response.errors
        );
    }

    #[tokio::test]
    async fn event_stream_yields_subscribed_kinds_only() {
        use tangible_objects_runtime::LifecycleEvent;
        use tokio_stream::StreamExt;

        let (mut runtime, handle) = EngineRuntime::new(Vec::new(), Vec::new());
        let schema = build_schema(handle);

        let mut stream = schema.execute_stream(
            "subscription { eventStream(kinds: [PROFILE_REGISTERED]) { kind name } }",
        );

        // Broadcast receivers only exist once the subscription stream is
        // polled - publish in a loop so the event lands after it registers.
        let publisher = tokio::spawn(async move {
            loop {
                runtime.publish(
                    &EngineFrame {
                        matches: Vec::new(),
                        touches: Vec::new(),
                        events: vec![EngineEvent::TouchPointsUpdated],
                        candidates: Vec::new(),
                        ran_generic: true,
                    },
                    (&[]).into(),
                );
                runtime.emit_lifecycle(LifecycleEvent::ProfileRegistered {
                    pattern_id: PatternId::new(3),
                    name: "fig".into(),
                    owner: None,
                });
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        });

        let response = tokio::time::timeout(std::time::Duration::from_secs(2), stream.next())
            .await
            .expect("event within timeout")
            .expect("a response item");
        publisher.abort();

        // only the requested kind arrives - TouchPointsUpdated was filtered
        assert!(response.errors.is_empty(), "errors: {:?}", response.errors);
        let data = response.data.into_json().expect("data as json");
        assert_eq!(data["eventStream"]["kind"], "PROFILE_REGISTERED");
        assert_eq!(data["eventStream"]["name"], "fig");
    }
}
