/// Live vs. training visualization - toggled by the control bar's
/// mode button.
#[derive(Clone, Copy, PartialEq)]
pub enum GuiMode {
    /// Detection view: matched figures + tracked poses.
    Live,
    /// Capture view: point set stabilizes for profile saving.
    Training,
}
