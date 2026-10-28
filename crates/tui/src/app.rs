//! App state + event loop - the subscription drives redraws, a 50 ms
//! ticker drains keyboard input and keeps the canvas fresh (TUI.md).

use std::collections::HashMap;
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
use tangible_objects_model::ContactCount;
use tangible_objects_model::EventKind;
use tangible_objects_model::FieldName;
use tangible_objects_model::FieldSet;
use tangible_objects_model::FieldSetName;
use tangible_objects_model::FilterMode;
use tangible_objects_model::ProfileInfo;
use tangible_objects_model::SessionMetrics;
use tangible_objects_model::TangibleEvent;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TrackedTangible;

use crate::EventLog;
use crate::Tab;
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
    /// Loaded figure profiles.
    profiles: Vec<ProfileInfo>,
    /// Latest session metrics (refreshed on a slow ticker).
    metrics: Option<SessionMetrics>,
    /// Registered occupancy field sets (`fieldSets` query).
    field_sets: Vec<FieldSet>,
    /// Last reported debounced contact count per `(set, field)`.
    field_counts: HashMap<(FieldSetName, FieldName), ContactCount>,
    /// Last known filter mode (set via `f` or `FILTER_MODE_CHANGED`).
    filter_mode: Option<FilterMode>,
    /// Active dashboard page.
    tab: Tab,
    /// Lifecycle/set/field event history for the log pane.
    log: EventLog,
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

/// Metrics pane refresh cadence - the counters are frame-rate data,
/// a 2 s poll keeps the pane live without spamming the daemon.
const METRICS_TICK: Duration = Duration::from_secs(2);

impl App {
    /// Wraps the client and seeds the surface.
    pub fn new(client: GraphQLClient) -> Self {
        Self {
            client,
            touches: Vec::new(),
            tangibles: Vec::new(),
            profiles: Vec::new(),
            metrics: None,
            field_sets: Vec::new(),
            field_counts: HashMap::new(),
            filter_mode: None,
            tab: Tab::Overview,
            log: EventLog::new(),
            bounds_mm: FALLBACK_MM,
            status: "connecting…".into(),
        }
    }

    /// Runs the draw/input loop until `q`/`Esc` or a stream error.
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), ClientError> {
        self.refresh_all().await;
        self.refresh_bounds().await;
        let query = EventStreamQuery {
            max_hz: Some(30),
            ..EventStreamQuery::all()
        };
        let mut events = self.client.event_stream(&query).await?;
        let mut metrics_tick = tokio::time::interval(METRICS_TICK);

        loop {
            terminal
                .draw(|frame| view::draw(frame, self))
                .map_err(io_to_client)?;
            tokio::select! {
                event = events.next() => {
                    match event {
                        Some(Ok(evt)) => {
                            self.on_event(&evt).await;
                        }
                        Some(Err(e)) => return Err(e),
                        None => return Ok(()),
                    }
                }
                _ = metrics_tick.tick() => {
                    self.refresh_metrics().await;
                }
                _ = tokio::time::sleep(TICK) => {
                    while event::poll(Duration::ZERO).unwrap_or(false) {
                        if let Event::Key(key) = event::read().unwrap_or(Event::Resize(0, 0)) {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                                KeyCode::Char('f') => self.cycle_filter_mode().await,
                                KeyCode::Char('r') => self.refresh_all().await,
                                KeyCode::Char('1') => self.tab = Tab::Overview,
                                KeyCode::Char('2') => self.tab = Tab::Live,
                                KeyCode::Tab | KeyCode::Right => self.tab = self.tab.next(),
                                KeyCode::Left | KeyCode::BackTab => self.tab = self.tab.prev(),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    /// Routes a streamed event: frame kinds refresh the snapshot,
    /// profile lifecycle refreshes the profile list, everything else
    /// lands in the log pane.
    async fn on_event(&mut self, evt: &TangibleEvent) {
        match evt.kind {
            EventKind::TouchPointsUpdated
            | EventKind::TangibleAdded
            | EventKind::TangibleMoved
            | EventKind::TangibleRemoved => {
                self.refresh_tangibles().await;
            }
            EventKind::ProfileRegistered | EventKind::ProfileUnregistered => {
                self.refresh_profiles().await;
                self.log.record(evt);
            }
            EventKind::FieldSetRegistered | EventKind::FieldSetUnregistered => {
                self.refresh_field_sets().await;
                self.log.record(evt);
            }
            EventKind::FieldPlaced | EventKind::FieldLifted | EventKind::FieldCountChanged => {
                self.record_field_count(evt);
                self.log.record(evt);
            }
            EventKind::FilterModeChanged => {
                if let Some(detail) = &evt.detail
                    && let Ok(mode) = detail.parse()
                {
                    self.filter_mode = Some(mode);
                }
                self.log.record(evt);
            }
            EventKind::Lagged => {
                self.log.record(evt);
                self.refresh_all().await;
            }
            _ => self.log.record(evt),
        }
    }

    /// Cycles the filter mode (`f` key): off → immediate → gated →
    /// guarded → tangibles-only.
    async fn cycle_filter_mode(&mut self) {
        let next = match self.filter_mode {
            None | Some(FilterMode::Off) => FilterMode::Immediate,
            Some(FilterMode::Immediate) => FilterMode::Gated,
            Some(FilterMode::Gated) => FilterMode::Guarded,
            Some(FilterMode::Guarded) => FilterMode::TangiblesOnly,
            Some(FilterMode::TangiblesOnly) => FilterMode::Off,
        };
        match self.client.set_filter_mode(next, None, None).await {
            Ok(true) => self.filter_mode = Some(next),
            Ok(false) | Err(_) => {
                self.status = format!("setFilterMode {next} rejected");
            }
        }
    }

    /// Full snapshot pull - touches, tangibles, profiles, metrics.
    async fn refresh_all(&mut self) {
        self.refresh_tangibles().await;
        self.refresh_profiles().await;
        self.refresh_field_sets().await;
        self.refresh_metrics().await;
    }

    /// Pulls the live `touchPoints` + `tangibles` into the snapshot.
    async fn refresh_tangibles(&mut self) {
        if let Ok(touches) = self.client.touch_points().await {
            self.touches = touches;
        }
        match self.client.tangibles().await {
            Ok(tangibles) => {
                self.tangibles = tangibles;
                self.status = format!(
                    "{} touch points, {} tangibles - f filter  r refresh  q quit",
                    self.touches.len(),
                    self.tangibles.len()
                );
            }
            Err(e) => {
                self.status = format!("daemon error: {e}");
            }
        }
    }

    /// Pulls the loaded profile list.
    async fn refresh_profiles(&mut self) {
        if let Ok(profiles) = self.client.profiles(&Default::default()).await {
            self.profiles = profiles;
        }
    }

    /// Pulls the session metrics.
    async fn refresh_metrics(&mut self) {
        if let Ok(metrics) = self.client.metrics().await {
            self.metrics = Some(metrics);
        }
    }

    /// Pulls the registered field sets.
    async fn refresh_field_sets(&mut self) {
        if let Ok(field_sets) = self.client.field_sets().await {
            self.field_sets = field_sets;
        }
    }

    /// Updates the last known contact count of the event's field:
    /// `FIELD_COUNT_CHANGED` carries the exact count, placed/lifted
    /// imply at least one/zero.
    fn record_field_count(&mut self, evt: &TangibleEvent) {
        let (Some(set), Some(field)) = (&evt.field_set, &evt.field) else {
            return;
        };
        let key = (set.clone(), field.clone());
        match evt.kind {
            EventKind::FieldCountChanged => {
                if let Some(count) = evt.count {
                    self.field_counts.insert(key, count);
                }
            }
            EventKind::FieldPlaced => {
                self.field_counts.insert(key, ContactCount::new(1));
            }
            EventKind::FieldLifted => {
                self.field_counts.insert(key, ContactCount::new(0));
            }
            _ => {}
        }
    }

    /// Reads surface bounds from `deviceInfo` (mm via axis ranges).
    async fn refresh_bounds(&mut self) {
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

    /// Latest touch points.
    pub fn touches(&self) -> &[TouchPoint] {
        &self.touches
    }

    /// Latest tracked tangibles.
    pub fn tangibles(&self) -> &[TrackedTangible] {
        &self.tangibles
    }

    /// Loaded figure profiles.
    pub fn profiles(&self) -> &[ProfileInfo] {
        &self.profiles
    }

    /// Latest session metrics.
    pub fn metrics(&self) -> Option<&SessionMetrics> {
        self.metrics.as_ref()
    }

    /// Registered occupancy field sets.
    pub fn field_sets(&self) -> &[FieldSet] {
        &self.field_sets
    }

    /// Last reported debounced contact count of a field.
    pub fn field_count(&self, set: &FieldSetName, field: &FieldName) -> Option<ContactCount> {
        self.field_counts
            .get(&(set.clone(), field.clone()))
            .copied()
    }

    /// Active dashboard page.
    pub fn tab(&self) -> Tab {
        self.tab
    }

    /// Last known filter mode.
    pub fn filter_mode(&self) -> Option<FilterMode> {
        self.filter_mode
    }

    /// Buffered event-log lines, oldest first.
    pub fn log_lines(&self) -> impl Iterator<Item = &String> {
        self.log.lines()
    }

    /// Surface bounds in mm.
    pub fn bounds(&self) -> (f64, f64) {
        self.bounds_mm
    }

    /// Status line text.
    pub fn status(&self) -> &str {
        &self.status
    }
}

/// Maps an `io::Error` into the client error type (terminal draw
/// failures surface like transport failures).
fn io_to_client(e: io::Error) -> ClientError {
    ClientError::Protocol(e.to_string())
}
