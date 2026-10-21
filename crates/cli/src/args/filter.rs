//! `filter` args - suppression policy switching.

/// `filter` args.
#[derive(Debug, clap::Args)]
pub struct FilterArgs {
    /// `off|immediate|gated|guarded|tangibles-only` (or `on` for
    /// `guarded`).
    pub mode: String,
    /// Override the stability gate (`0` keeps the default).
    #[arg(long)]
    pub stability: Option<u64>,
    /// Apply to one stage only (default: all).
    #[arg(long)]
    pub stage: Option<String>,
}
