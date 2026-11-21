//! `spells` args - multi-finger gesture registration and training.

use clap::Subcommand;
use std::path::PathBuf;
use tangible_objects_model::Lifetime;
use tangible_objects_model::Owner;
use tangible_objects_model::SpellArming;
use tangible_objects_model::SpellName;

/// `spells` subcommands.
#[derive(Debug, clap::Args)]
pub struct SpellsArgs {
    /// The spell operation.
    #[command(subcommand)]
    pub command: SpellsCommand,
}

/// `spells` subcommands.
#[derive(Debug, Subcommand)]
pub enum SpellsCommand {
    /// Register a spell file (`registerSpell`).
    Register {
        /// Path to the spell `.toml`.
        file: PathBuf,
        /// Owning application.
        #[arg(long)]
        owner: Option<Owner>,
        /// `session` (default) or `persistent`.
        #[arg(long, default_value = "session")]
        lifetime: Lifetime,
    },
    /// Remove a registered spell (`unregisterSpell`).
    Unregister {
        /// Spell name.
        name: SpellName,
    },
    /// Ask the daemon to train a spell (`trainSpell`) - draw the
    /// gesture 3 times; the finished spell registers persistently.
    Train {
        /// Spell name.
        name: SpellName,
        /// Arming the finished spell carries - the `[arming]` table
        /// as TOML (e.g. 'type = "five-finger-tap"' or
        /// 'type = "start-spot"' plus `[region]`);
        /// defaults to `free`.
        #[arg(long)]
        arming: Option<SpellArming>,
    },
}
