//! `record` args - recording session control.

use clap::Subcommand;
use std::path::PathBuf;
use tangible_objects_model::RecordLabel;

/// `record` subcommands.
#[derive(Debug, clap::Args)]
pub struct RecordArgs {
    /// The recording operation.
    #[command(subcommand)]
    pub command: RecordCommand,
}

/// `record` subcommands.
#[derive(Debug, Subcommand)]
pub enum RecordCommand {
    /// Session status (`recordingStatus` query).
    Status,
    /// Start a session (`startRecording`).
    Start {
        /// Artifact output path on the daemon host.
        path: PathBuf,
        /// Optional ground-truth label.
        #[arg(long)]
        label: Option<RecordLabel>,
    },
    /// Stop the session (`stopRecording`).
    Stop,
    /// Set the label mid-session (`setRecordLabel`).
    Label {
        /// Label text; omit with `--clear` to unset.
        label: Option<RecordLabel>,
        /// Clear the label.
        #[arg(long)]
        clear: bool,
    },
}
