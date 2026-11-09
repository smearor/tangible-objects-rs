use tangible_objects_model::DisplayRotation;
use tangible_objects_model::FilterMode;

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
    /// Whether the edge-margin overlay starts drawn
    /// (`SetEdgeMarginVisible` commands toggle afterwards).
    pub edge_margin_visible: bool,
    /// Whether the field-set overlay starts drawn
    /// (`SetFieldsVisible` commands toggle afterwards).
    pub fields_visible: bool,
    /// Whether the ruler overlay starts drawn
    /// (`SetRulerVisible` commands toggle afterwards).
    pub ruler_visible: bool,
    /// Whether the grid overlay starts drawn
    /// (`SetGridVisible` commands toggle afterwards).
    pub grid_visible: bool,
    /// Whether the positions overlay starts drawn
    /// (`SetPositionsVisible` commands toggle afterwards).
    pub positions_visible: bool,
    /// Whether touches pass through the window to the desktop except
    /// on GUI elements (`SetTouchPassthrough` commands toggle
    /// afterwards; Wayland only).
    pub touch_passthrough: bool,
    /// Initial filter mode shown in the GUI (`--filter-mode`) -
    /// display state, honored by the proxy in `--filter` mode.
    pub filter_mode: FilterMode,
}
