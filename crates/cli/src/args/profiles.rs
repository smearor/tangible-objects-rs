//! `profiles` args - profile listing, registration and training.

use clap::Subcommand;
use std::path::PathBuf;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::PatternName;

/// `profiles` subcommands.
#[derive(Debug, clap::Args)]
pub struct ProfilesArgs {
    /// The profile operation.
    #[command(subcommand)]
    pub command: ProfilesCommand,
}

/// `profiles` subcommands.
#[derive(Debug, Subcommand)]
pub enum ProfilesCommand {
    /// List profiles (`--owner`/`--static-only` narrow).
    List {
        /// Only profiles owned by this application.
        #[arg(long)]
        owner: Option<Owner>,
        /// Only statically loaded profiles.
        #[arg(long)]
        static_only: bool,
    },
    /// Show one profile by pattern ID.
    Show {
        /// Pattern ID.
        id: PatternId,
    },
    /// Remove a registered pattern (`unregisterPattern`).
    Delete {
        /// Pattern ID.
        id: PatternId,
    },
    /// Drop every registration of an owner (`unregisterOwner`).
    UnregisterOwner {
        /// Owner string.
        owner: Owner,
    },
    /// Register a profile file at runtime (`registerPattern`).
    Register {
        /// Path to the profile `.toml`.
        file: PathBuf,
        /// Owning application.
        #[arg(long)]
        owner: Option<Owner>,
        /// `session` (default) or `persistent`.
        #[arg(long, default_value = "session")]
        lifetime: Lifetime,
    },
    /// Replace a pattern's geometry (`updateProfile`).
    Update {
        /// Pattern ID.
        id: PatternId,
        /// Path to the profile `.toml`.
        file: PathBuf,
    },
    /// Ask the daemon to record a profile (`trainProfile`).
    Train {
        /// Figure name.
        name: PatternName,
    },
}
