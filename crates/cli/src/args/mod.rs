//! `clap` argument tree - [`Cli`] + the [`Command`] enum live here;
//! each topic's args/command types sit in a matching file.

mod analyze;
mod completion;
mod field_sets;
mod filter;
mod message;
mod metrics;
mod profiles;
mod record;
mod stages;
mod watch;

use clap::Parser;
use clap::Subcommand;

pub use analyze::AnalyzeArgs;
pub use analyze::AnalyzeCommand;
pub use completion::CompletionArgs;
pub use field_sets::FieldSetsArgs;
pub use field_sets::FieldSetsCommand;
pub use filter::FilterArgs;
pub use message::MessageArgs;
pub use metrics::MetricsArgs;
pub use metrics::MetricsCommand;
pub use profiles::ProfilesArgs;
pub use profiles::ProfilesCommand;
pub use record::RecordArgs;
pub use record::RecordCommand;
pub use stages::StagesArgs;
pub use stages::StagesCommand;
pub use watch::WatchArgs;

/// CLI client for the tangible-objects daemon's GraphQL endpoint.
#[derive(Debug, Parser)]
#[command(name = "tangible-objects-cli", version, about)]
pub struct Cli {
    /// Daemon endpoint (flag > `TANGIBLE_ENDPOINT` > client.toml >
    /// `http://127.0.0.1:8080/graphql`).
    #[arg(long, env = "TANGIBLE_ENDPOINT")]
    pub endpoint: Option<String>,
    /// Emit payloads as pretty JSON instead of the line format.
    #[arg(long, global = true)]
    pub json: bool,
    /// The command to run.
    #[command(subcommand)]
    pub command: Command,
}

/// Top-level subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Input device info (name + axis calibration).
    Devices,
    /// Session overview: device, recording, metrics summary.
    Status,
    /// Full session metrics (JSON).
    Metrics(MetricsArgs),
    /// User-facing message shown on the panel UI.
    Message(MessageArgs),
    /// Currently tracked tangibles (live poses).
    Tangibles,
    /// Active touch points.
    Touches,
    /// Figure profile operations.
    Profiles(ProfilesArgs),
    /// Occupancy field-set operations.
    FieldSets(FieldSetsArgs),
    /// Pipeline stage operations.
    Stages(StagesArgs),
    /// Switch the suppression policy.
    Filter(FilterArgs),
    /// Completion configuration.
    Completion(CompletionArgs),
    /// Recording session control.
    Record(RecordArgs),
    /// Offline analysis queries.
    Analyze(AnalyzeArgs),
    /// Live event stream.
    Watch(WatchArgs),
}
