/// Live vs. training visualization - toggled by the control bar's
/// mode button.
#[derive(Clone, Copy, PartialEq)]
pub enum GuiMode {
    /// Detection view: matched figures + tracked poses.
    Live,
    /// Capture view: point set stabilizes for profile saving.
    Training,
    /// Spell-capture view (SPELLS.md): gesture episodes are recorded
    /// as template repetitions for a new spell - the engine keeps
    /// processing frames so the trajectory/debug overlays stay live.
    SpellTraining,
}
