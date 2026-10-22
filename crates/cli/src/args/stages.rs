//! `stages` args - pipeline stage lifecycle.

use clap::Subcommand;
use tangible_objects_model::StageName;

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
        name: StageName,
    },
    /// Remove a stage (`unregisterStage`).
    Unregister {
        /// Stage name.
        name: StageName,
    },
}
