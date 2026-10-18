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
    use tangible_objects_model::PatternId;
    use tangible_objects_model::RotationRad;
    use tangible_objects_model::TangibleId;
    use tangible_objects_model::TrackedTangible;
    use tangible_objects_model::Vec2;
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
            pending_frames: 0,
            spawn_stability: tangible_objects_model::FrameCount::new(15),
            frames_since_match: 0,
        }]
    }

    #[tokio::test]
    async fn tangibles_query_reads_the_snapshot() {
        let (mut runtime, handle) = EngineRuntime::new(
            Vec::new(),
            vec![ProfileInfo {
                pattern_id: PatternId::new(1),
                name: "triangle".into(),
                num_points: 3,
                owner: None,
                spawn_on_partial: false,
                spawn_stability: tangible_objects_model::FrameCount::new(15),
            }],
        );
        let schema = build_schema(handle);

        runtime.publish(
            &EngineFrame {
                matches: Vec::new(),
                touches: Vec::new(),
                events: vec![EngineEvent::TangibleAdded(tracked()[0].clone())],
                candidates: Vec::new(),
            },
            &tracked(),
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
            Ok(tangible_objects_runtime::ControlCommand::SetFilterMode {
                mode: tangible_objects_model::FilterMode::Guarded,
                stability: Some(_),
            })
        ));
    }
}
