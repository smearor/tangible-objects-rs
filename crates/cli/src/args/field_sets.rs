//! `field-sets` args - occupancy field-set registration.

use clap::Subcommand;
use std::path::PathBuf;
use tangible_objects_model::FieldSetName;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;

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
        file: PathBuf,
        /// Owning application.
        #[arg(long)]
        owner: Option<Owner>,
        /// `session` (default) or `persistent`.
        #[arg(long)]
        lifetime: Option<Lifetime>,
    },
    /// Remove a field set (`unregisterFieldSet`).
    Unregister {
        /// Field-set name.
        name: FieldSetName,
    },
}
