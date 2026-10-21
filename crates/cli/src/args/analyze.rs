//! `analyze` args - offline analysis queries.

use clap::Subcommand;

/// `analyze` subcommands.
#[derive(Debug, clap::Args)]
pub struct AnalyzeArgs {
    /// The analysis operation.
    #[command(subcommand)]
    pub command: AnalyzeCommand,
}

/// `analyze` subcommands.
#[derive(Debug, Subcommand)]
pub enum AnalyzeCommand {
    /// Deviation analysis over a recording (`analyzeRecording`).
    Recording {
        /// Recording path (`.jsonl`/`.evtest`/`.dataset.jsonl`).
        path: String,
        /// `candidates` (default) or `contacts`.
        #[arg(long)]
        mode: Option<String>,
        /// Profiles directory for `contacts` mode.
        #[arg(long)]
        profiles_dir: Option<String>,
    },
    /// Pairwise profile-collision report (`analyzeSimilarity`).
    Similarity {
        /// Profiles directory.
        #[arg(long)]
        profiles_dir: Option<String>,
    },
    /// Fetch a recording artifact's content (`recordingArtifact`).
    Artifact {
        /// Artifact path.
        path: String,
    },
}
