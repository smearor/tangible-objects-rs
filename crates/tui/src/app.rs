//! App state + event loop - the subscription drives redraws, a 50 ms
//! ticker drains keyboard input and keeps the canvas fresh.

use std::io;
use std::time::Duration;

use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use futures_lite::StreamExt;
use ratatui::Terminal;
use ratatui::backend::Backend;
use tangible_objects_client::ClientError;
use tangible_objects_client::EventStreamQuery;
use tangible_objects_client::GraphQLClient;
use tangible_objects_model::EventKind;
use tangible_objects_model::StreamEvent;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TrackedTangible;

use crate::view;

/// Live-view dashboard state - the rendered snapshot plus the surface
/// bounds the canvas maps millimeters into.
pub struct App {
    /// Typed daemon client.
    client: GraphQLClient,
    /// Latest touch points.
    touches: Vec<TouchPoint>,
    /// Latest tracked tangibles.
    tangibles: Vec<TrackedTangible>,
    /// Surface bounds in mm (from `deviceInfo`, fallback square).
    bounds_mm: (f64, f64),
    /// Status line shown under the canvas.
    status: String,
}

/// Millimeters per fallback surface when the daemon reports no
/// calibration.
const FALLBACK_MM: (f64, f64) = (500.0, 400.0);

/// Keyboard event poll cadence.
const TICK: Duration = Duration::from_millis(50);

impl App {
    /// Wraps the client and seeds the surface.
    pub fn new(client: GraphQLClient) -> Self {
        Self {
            client,
            touches: Vec::new(),
            tangibles: Vec::new(),
            bounds_mm: FALLBACK_MM,
            status: "connecting…".into(),
        }
    }

    /// Runs the draw/input loop until `q`/`Esc` or a stream error.
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), ClientError> {
        self.refresh().await;
        self.bounds().await;
        let query = EventStreamQuery {
            kinds: vec![
                StreamEvent::TouchPointsUpdated,
                StreamEvent::TangibleAdded,
                StreamEvent::TangibleMoved,
                StreamEvent::TangibleRemoved,
            ],
            pattern_id: None,
            set_id: None,
            max_hz: Some(30),
        };
        let mut events = self.client.event_stream(&query).await?;

        loop {
            terminal
                .draw(|frame| view::draw(frame, self))
                .map_err(io_to_client)?;
            tokio::select! {
                event = events.next() => {
                    match event {
                        Some(Ok(evt)) => {
                            self.on_event(&evt.kind).await;
                        }
                        Some(Err(e)) => return Err(e),
                        None => return Ok(()),
                    }
                }
                _ = tokio::time::sleep(TICK) => {
                    while event::poll(Duration::ZERO).unwrap_or(false) {
                        if let Event::Key(key) = event::read().unwrap_or(Event::Resize(0, 0))
                            && (key.code == KeyCode::Char('q') || key.code == KeyCode::Esc) {
                                return Ok(());
                            }
                    }
                }
            }
        }
    }

    /// Re-fetches touches + tangibles when a relevant kind arrives.
    async fn on_event(&mut self, kind: &EventKind) {
        match kind {
            EventKind::TouchPointsUpdated
            | EventKind::TangibleAdded
            | EventKind::TangibleMoved
            | EventKind::TangibleRemoved => self.refresh().await,
            _ => {}
        }
    }

    /// Pulls the live `touchPoints` + `tangibles` into the snapshot.
    async fn refresh(&mut self) {
        if let Ok(touches) = self.client.touch_points().await {
            self.touches = touches;
        }
        match self.client.tangibles().await {
            Ok(tangibles) => {
                self.tangibles = tangibles;
                self.status = format!(
                    "{} touch points, {} tangibles - q to quit",
                    self.touches.len(),
                    self.tangibles.len()
                );
            }
            Err(e) => {
                self.status = format!("daemon error: {e}");
            }
        }
    }

    /// Reads surface bounds from `deviceInfo` (mm via axis ranges).
    async fn bounds(&mut self) {
        if let Ok(Some(info)) = self.client.device_info().await
            && let (Some(x_res), Some(y_res), Some(x_min), Some(x_max), Some(y_min), Some(y_max)) = (
                info.x_res, info.y_res, info.x_min, info.x_max, info.y_min, info.y_max,
            )
        {
            self.bounds_mm = (
                (x_max.as_f64() - x_min.as_f64()) / x_res.as_f64(),
                (y_max.as_f64() - y_min.as_f64()) / y_res.as_f64(),
            );
        }
    }

    /// Accessors for the view.
    pub(crate) fn state(&self) -> (&[TouchPoint], &[TrackedTangible], (f64, f64), &str) {
        (&self.touches, &self.tangibles, self.bounds_mm, &self.status)
    }
}

/// Maps an `io::Error` into the client error type (terminal draw
/// failures surface like transport failures).
fn io_to_client(e: io::Error) -> ClientError {
    ClientError::Protocol(e.to_string())
}
