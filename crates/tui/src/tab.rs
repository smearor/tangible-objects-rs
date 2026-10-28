//! `Tab` - the dashboard pages (TUI.md): an occupancy overview and
//! the live surface view.

/// Which page the TUI shows. `1`/`2` select, `Tab`/`←`/`→` cycle.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Tab {
    /// Slot/tangible/field/field-set occupancy columns.
    Overview,
    /// Braille surface canvas + profiles/metrics/log panes.
    Live,
}

impl Tab {
    /// Titles rendered in the tab bar, in page order.
    pub const TITLES: [&'static str; 2] = ["[1] overview", "[2] live"];

    /// Renders a key in the `Tabs` widget.
    pub fn index(self) -> usize {
        match self {
            Self::Overview => 0,
            Self::Live => 1,
        }
    }

    /// `Tab`/`→` key.
    pub fn next(self) -> Self {
        match self {
            Self::Overview => Self::Live,
            Self::Live => Self::Overview,
        }
    }

    /// `←` key.
    pub fn prev(self) -> Self {
        self.next()
    }
}
