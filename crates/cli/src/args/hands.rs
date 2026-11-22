//! `hands` args - handprint registration (SPELLS.md's per-user
//! spells).

use clap::Subcommand;
use std::path::PathBuf;
use tangible_objects_model::HandName;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;

/// `hands` subcommands.
#[derive(Debug, clap::Args)]
pub struct HandsArgs {
    /// The hand operation.
    #[command(subcommand)]
    pub command: HandsCommand,
}

/// `hands` subcommands.
#[derive(Debug, Subcommand)]
pub enum HandsCommand {
    /// Register a hand profile file (`registerHandProfile`).
    Register {
        /// Path to the hand profile `.toml`.
        file: PathBuf,
        /// Owning application.
        #[arg(long)]
        owner: Option<Owner>,
        /// `session` (default) or `persistent`.
        #[arg(long, default_value = "session")]
        lifetime: Lifetime,
    },
    /// Remove a registered hand profile (`unregisterHandProfile`).
    Unregister {
        /// Hand name.
        name: HandName,
    },
}
