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
    use smearor_commons_geometry::RotationRad;
    use smearor_commons_geometry::Vec2;
    use tangible_objects_engine::EngineEvent;
    use tangible_objects_engine::EngineFrame;
    use tangible_objects_model::ContactCount;
    use tangible_objects_model::DEFAULT_SPAWN_STABILITY;
    use tangible_objects_model::DeviceInfo;
    use tangible_objects_model::FilterMode;
    use tangible_objects_model::FrameCount;
    use tangible_objects_model::PatternId;
    use tangible_objects_model::PinCount;
    use tangible_objects_model::SlotSet;
    use tangible_objects_model::TangibleId;
    use tangible_objects_model::TrackedTangible;
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

    fn frame(events: Vec<EngineEvent>) -> EngineFrame {
        EngineFrame {
            matches: Vec::new(),
            touches: Vec::new(),
            events,
            candidates: Vec::new(),
            ran_generic: true,
            candidates_tested: 0.into(),
            dead_zone_filtered: ContactCount::default(),
            spell_matches: Vec::new(),
            gesture_slots: SlotSet::default(),
            consumed_slots: SlotSet::default(),
        }
    }

    /// Applies the next control command the way the daemon's device
    /// loop does - mutations wait on the command's reply oneshot, so
    /// the test drains the channel while the GraphQL call is pending.
    async fn apply_next_command(runtime: &mut EngineRuntime) {
        for _ in 0..200 {
            match runtime.try_recv_command() {
                Ok(ControlCommand::ClaimSeatSession { name, owner, reply }) => {
                    let _ = reply.send(runtime.claim_seat_session(name, owner));
                    return;
                }
                Ok(ControlCommand::ReleaseSeatSession { name, owner, reply }) => {
                    let _ = reply.send(runtime.release_seat_session(name, owner));
                    return;
                }
                Ok(ControlCommand::AddSeat(request)) => {
                    let request = *request;
                    let _ = request.reply.send(runtime.add_seat(
                        request.session,
                        request.owner,
                        request.name,
                        request.marker,
                        request.pose,
                    ));
                    return;
                }
                Ok(_) | Err(_) => tokio::time::sleep(std::time::Duration::from_millis(2)).await,
            }
        }
        panic!("no control command arrived within the drain window");
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
                enabled: true,
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
                candidates_tested: 0.into(),
                dead_zone_filtered: ContactCount::default(),
                spell_matches: Vec::new(),
                gesture_slots: SlotSet::default(),
                consumed_slots: SlotSet::default(),
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
                        candidates_tested: 0.into(),
                        dead_zone_filtered: ContactCount::default(),
                        spell_matches: Vec::new(),
                        gesture_slots: SlotSet::default(),
                        consumed_slots: SlotSet::default(),
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

    /// Claim -> addSeat -> marker binds -> seatSessions query ->
    /// releaseSeatSession - the interface roundtrip from
    /// TANGIBLE_OBJECTS_TABLE_TOP_SEAT.md's phase-2 exit criteria.
    #[tokio::test]
    async fn seat_session_roundtrip_claim_add_query_release() {
        let (mut runtime, handle) = EngineRuntime::new(Vec::new(), Vec::new());
        runtime.set_device_info(DeviceInfo {
            x_res: Some(1.0.into()),
            y_res: Some(1.0.into()),
            x_min: Some(0.0.into()),
            x_max: Some(400.0.into()),
            y_min: Some(0.0.into()),
            y_max: Some(300.0.into()),
            ..Default::default()
        });
        let schema = build_schema(handle);

        // The mutation parks on the command's reply oneshot - drain the
        // control channel like the device loop while it is pending.
        let claim = {
            let schema = schema.clone();
            tokio::spawn(async move {
                schema
                    .execute(
                        r#"mutation {
                            claimSeatSession(name: "game", owner: "app") {
                                __typename
                                ... on SeatSessionClaimed { session { name owner } }
                                ... on RegistrationRejected { kind }
                            }
                        }"#,
                    )
                    .await
            })
        };
        apply_next_command(&mut runtime).await;
        let result = claim.await.expect("claim task");
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        let data = result.data.into_json().expect("data as json");
        assert_eq!(data["claimSeatSession"]["__typename"], "SeatSessionClaimed");

        // Marker-backed seat (profile 1) - marker validation against
        // registered patterns lives in the daemon arm, the runtime
        // applies whatever profile ID arrives.
        let add = {
            let schema = schema.clone();
            tokio::spawn(async move {
                schema
                    .execute(
                        r#"mutation {
                            addSeat(
                                input: { session: "game", name: "seat-1", marker: 1 }
                                owner: "app"
                            ) {
                                __typename
                                ... on SeatAdded { seat { name state } }
                                ... on RegistrationRejected { kind }
                            }
                        }"#,
                    )
                    .await
            })
        };
        apply_next_command(&mut runtime).await;
        let result = add.await.expect("add task");
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        let data = result.data.into_json().expect("data as json");
        assert_eq!(data["addSeat"]["__typename"], "SeatAdded");
        assert_eq!(data["addSeat"]["seat"]["state"], "ABSENT");

        // A tracked marker instance binds on the next published frame
        // and the pose derives from its center/rotation.
        runtime.publish(&frame(Vec::new()), (&tracked()).into());

        // The query serves the live registry - no extra publish needed.
        let result = schema
            .execute(
                r#"{ seatSessions { name owner seats { name state tangible pose { orientation } } } }"#,
            )
            .await;
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        let data = result.data.into_json().expect("data as json");
        assert_eq!(data["seatSessions"][0]["name"], "game");
        assert_eq!(data["seatSessions"][0]["seats"][0]["name"], "seat-1");
        assert_eq!(data["seatSessions"][0]["seats"][0]["state"], "ANCHORED");
        assert_eq!(data["seatSessions"][0]["seats"][0]["tangible"], 7);

        // Release removes the session and its roster.
        let release = {
            let schema = schema.clone();
            tokio::spawn(async move {
                schema
                    .execute(
                        r#"mutation {
                            releaseSeatSession(name: "game", owner: "app") {
                                __typename
                                ... on SeatSessionReleased { session { name } }
                                ... on RegistrationRejected { kind }
                            }
                        }"#,
                    )
                    .await
            })
        };
        apply_next_command(&mut runtime).await;
        let result = release.await.expect("release task");
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        let data = result.data.into_json().expect("data as json");
        assert_eq!(
            data["releaseSeatSession"]["__typename"],
            "SeatSessionReleased"
        );

        let result = schema.execute("{ seatSessions { name } }").await;
        let data = result.data.into_json().expect("data as json");
        assert_eq!(data["seatSessions"].as_array().map(Vec::len), Some(0));
    }
}
