//! `completion` args - missing-pin completion configuration.

/// `completion` args - `null` fields stay unchanged.
#[derive(Debug, clap::Args)]
pub struct CompletionArgs {
    /// `match`/`exact`/`partial`/`off` (server's mode names).
    #[arg(long)]
    pub mode: Option<String>,
    /// Completion radius (mm).
    #[arg(long)]
    pub radius: Option<f64>,
    /// Max missing pins (0 = completion off).
    #[arg(long)]
    pub max_missing: Option<i64>,
    /// Persist the config back to disk.
    #[arg(long)]
    pub persist: Option<bool>,
}
