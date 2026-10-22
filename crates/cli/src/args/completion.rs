//! `completion` args - missing-pin completion configuration.

use tangible_objects_model::CompletionMode;

/// `completion` args - `null` fields stay unchanged.
#[derive(Debug, clap::Args)]
pub struct CompletionArgs {
    /// `match` or `exact`.
    #[arg(long)]
    pub mode: Option<CompletionMode>,
    /// Completion radius as a multiple of the pattern tolerance.
    #[arg(long)]
    pub radius: Option<f64>,
    /// Max missing pins (0 = completion off).
    #[arg(long)]
    pub max_missing: Option<i64>,
    /// Persist the config back to disk.
    #[arg(long)]
    pub persist: Option<bool>,
}
