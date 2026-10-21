//! `profiles` args - profile listing, registration and training.

use clap::Subcommand;

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
        owner: Option<String>,
        /// Only statically loaded profiles.
        #[arg(long)]
        static_only: bool,
    },
    /// Show one profile by pattern ID.
    Show {
        /// Pattern ID.
        id: u32,
    },
    /// Remove a registered pattern (`unregisterPattern`).
    Delete {
        /// Pattern ID.
        id: u32,
    },
    /// Drop every registration of an owner (`unregisterOwner`).
    UnregisterOwner {
        /// Owner string.
        owner: String,
    },
    /// Register a profile file at runtime (`registerPattern`).
    Register {
        /// Path to the profile `.toml`.
        file: String,
        /// Owning application.
        #[arg(long)]
        owner: Option<String>,
        /// `session` (default) or `persistent`.
        #[arg(long, default_value = "session")]
        lifetime: String,
    },
    /// Replace a pattern's geometry (`updateProfile`).
    Update {
        /// Pattern ID.
        id: u32,
        /// Path to the profile `.toml`.
        file: String,
    },
    /// Ask the daemon to record a profile (`trainProfile`).
    Train {
        /// Figure name.
        name: String,
    },
}
