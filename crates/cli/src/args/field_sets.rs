//! `field-sets` args - occupancy field-set registration.

use clap::Subcommand;

/// `field-sets` subcommands.
#[derive(Debug, clap::Args)]
pub struct FieldSetsArgs {
    /// The field-set operation.
    #[command(subcommand)]
    pub command: FieldSetsCommand,
}

/// `field-sets` subcommands.
#[derive(Debug, Subcommand)]
pub enum FieldSetsCommand {
    /// Register a field-set file (`registerFieldSet`).
    Register {
        /// Path to the field-set `.toml`.
        file: String,
    },
    /// Remove a field set (`unregisterFieldSet`).
    Unregister {
        /// Field-set name.
        name: String,
    },
}
