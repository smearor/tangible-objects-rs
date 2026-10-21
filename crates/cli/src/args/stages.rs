//! `stages` args - pipeline stage lifecycle.

use clap::Subcommand;

/// `stages` subcommands.
#[derive(Debug, clap::Args)]
pub struct StagesArgs {
    /// The stage operation.
    #[command(subcommand)]
    pub command: StagesCommand,
}

/// `stages` subcommands.
#[derive(Debug, Subcommand)]
pub enum StagesCommand {
    /// Claim/create a pipeline stage (`registerStage`).
    Register {
        /// Stage name.
        name: String,
    },
    /// Remove a stage (`unregisterStage`).
    Unregister {
        /// Stage name.
        name: String,
    },
}
