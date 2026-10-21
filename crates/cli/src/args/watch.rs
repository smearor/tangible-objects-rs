//! `watch` args - the `eventStream` filters verbatim.

/// `watch` args.
#[derive(Debug, clap::Args)]
pub struct WatchArgs {
    /// Comma-separated `StreamEvent` kinds (empty = `tangibleEvents`).
    #[arg(long)]
    pub kinds: Option<String>,
    /// Narrow to one pattern.
    #[arg(long)]
    pub pattern_id: Option<u32>,
    /// Narrow to one set.
    #[arg(long)]
    pub set_id: Option<u32>,
    /// Throttle `TOUCH_POINTS_UPDATED`.
    #[arg(long)]
    pub max_hz: Option<i32>,
    /// Stream `fieldEvents` instead, optionally narrowed to a set.
    #[arg(long)]
    pub fields: Option<Option<String>>,
    /// Deprecated convenience stream (`tangibleEvents`).
    #[arg(long)]
    pub tangibles: bool,
}
