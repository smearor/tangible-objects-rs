//! `metrics` args - the session-metrics surface.

use clap::Subcommand;

/// `metrics` sub-args.
#[derive(Debug, clap::Args)]
pub struct MetricsArgs {
    /// Reset the session counters.
    #[command(subcommand)]
    pub command: Option<MetricsCommand>,
}

/// `metrics` subcommands.
#[derive(Debug, Subcommand)]
pub enum MetricsCommand {
    /// Reset the session counters (`resetMetrics`).
    Reset,
}
