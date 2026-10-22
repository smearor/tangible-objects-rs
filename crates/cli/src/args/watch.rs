//! `watch` args - live event subscription.

use tangible_objects_model::FieldSetName;
use tangible_objects_model::PatternId;
use tangible_objects_model::StreamEvent;
use tangible_objects_model::TangibleSetId;

/// `watch` args - `eventStream`/`fieldEvents` subscriptions.
#[derive(Debug, clap::Args)]
pub struct WatchArgs {
    /// Event kinds (comma-separated, e.g.
    /// `tangible_added,set_added`; empty = tangible events).
    #[arg(long, value_delimiter = ',')]
    pub kinds: Vec<StreamEvent>,
    /// Restrict to one pattern.
    #[arg(long)]
    pub pattern_id: Option<PatternId>,
    /// Restrict to one set.
    #[arg(long)]
    pub set_id: Option<TangibleSetId>,
    /// Client-side throttle hint (Hz).
    #[arg(long)]
    pub max_hz: Option<i32>,
    /// Field-event subscription; `--fields` alone subscribes all sets,
    /// `--fields <set>` one set.
    #[arg(long, num_args = 0..=1, default_missing_value = None)]
    pub fields: Option<Option<FieldSetName>>,
    /// Shorthand for `kinds = TANGIBLE`.
    #[arg(long)]
    pub tangibles: bool,
}
