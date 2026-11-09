use tangible_objects_model::DisplayRotation;

use crate::BackgroundAlpha;

/// Display-side options for [`crate::run_gui`] - window presentation,
/// not detection behavior (`DetectionConfig` stays on the engine side).
#[derive(Debug, Copy, Clone)]
pub struct GuiOptions {
    /// Whether the window starts visible (`SetGuiVisible` toggles
    /// afterwards).
    pub visible: bool,
    /// Quarter turns applied when mapping surface millimeters onto
    /// screen pixels - the panel's native landscape frame stays in the
    /// evdev stream even when the desktop shows the screen rotated
    /// (e.g. GNOME portrait mode).
    pub rotation: DisplayRotation,
    /// Canvas background opacity. Below opaque the window shows the
    /// desktop beneath the touch visualization - the touches, figures
    /// and UI chrome stay opaque; only the black fill translucents
    /// (`cairo` alpha, not window opacity).
    pub background_alpha: BackgroundAlpha,
    /// Whether the dead-zone overlay starts drawn
    /// (`SetDeadZonesVisible` commands toggle afterwards).
    pub dead_zones_visible: bool,
}
