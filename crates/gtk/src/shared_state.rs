//! The render model shared between the device thread (writer) and the
//! GTK main loop (reader) - the GUI's entire cross-thread contract.

use std::sync::Arc;
use std::sync::Mutex;

use tangible_objects_model::Calibration;
use tangible_objects_model::PatternMatch;
use tangible_objects_model::TouchPoint;
use tangible_objects_model::TrackedTangible;

use crate::gui_mode::GuiMode;

/// Snapshot the draw functions consume - written by the device thread
/// once per `SYN_REPORT` frame, read by the ~30 fps GTK timer.
pub struct SharedState {
    /// Active touch points.
    pub touches: Vec<TouchPoint>,
    /// Raw pattern matches of the last frame (live view).
    pub matches: Vec<PatternMatch>,
    /// Currently tracked tangibles (pending flags render dimmed).
    pub tracked: Vec<TrackedTangible>,
    /// Effective axis calibration (mm conversion for the canvas scale).
    pub calibration: Option<Calibration>,
    /// Consecutive frames the contact set held the same size.
    pub stable_frames: u32,
    /// Contact set stable long enough for profile capture.
    pub stable: bool,
    /// Live vs. training visualization.
    pub mode: GuiMode,
    /// Device-open failure shown instead of the surface.
    pub device_error: Option<String>,
    /// User-facing message pushed via an interface (`ShowMessage`).
    pub message: Option<String>,
    /// Whether the window should be shown (`SetGuiVisible`).
    pub visible: bool,
}

impl SharedState {
    /// Initial state - hidden-only devices start visible (the
    /// `--gui` flag semantics), `SetGuiVisible` toggles afterwards.
    pub fn new(visible: bool) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            touches: Vec::new(),
            matches: Vec::new(),
            tracked: Vec::new(),
            calibration: None,
            stable_frames: 0,
            stable: false,
            mode: GuiMode::Live,
            device_error: None,
            message: None,
            visible,
        }))
    }
}

/// Whether the shared state is in live mode.
pub fn mode_is_live(state: &Arc<Mutex<SharedState>>) -> bool {
    state
        .lock()
        .map(|s| s.mode == GuiMode::Live)
        .unwrap_or(true)
}
