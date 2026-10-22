//! `filter` args - suppression policy switching.

use tangible_objects_model::FilterMode;
use tangible_objects_model::FilterModeParseError;
use tangible_objects_model::FrameCount;
use tangible_objects_model::StageName;

/// `filter` args - stage-scoped or broadcast.
#[derive(Debug, clap::Args)]
pub struct FilterArgs {
    /// `on`, `off`, `immediate`, `gated`, `guarded` or `tangibles-only`.
    #[arg(value_parser = parse_filter_mode)]
    pub mode: FilterMode,
    /// Consecutive matched frames before suppression engages
    /// (gated/guarded).
    #[arg(long)]
    pub stability: Option<FrameCount>,
    /// Scope to one pipeline stage; omit to apply to all.
    #[arg(long)]
    pub stage: Option<StageName>,
}

/// Parses `filter on|off|<mode>` - `on` is a CLI-only alias for
/// `guarded`, the rest delegates to `FilterMode`'s `FromStr`.
fn parse_filter_mode(text: &str) -> Result<FilterMode, FilterModeParseError> {
    match text {
        "on" => Ok(FilterMode::Guarded),
        other => other.parse(),
    }
}
