//! `SetFilterModeVariables` - the variables of `setFilterMode`.

use cynic::QueryVariables;
use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::StageName;

/// Variables of `setFilterMode` - mode required, stability/stage
/// optional.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct SetFilterModeVariables {
    /// Suppression mode.
    pub mode: FilterMode,
    /// Confirmation frames; `None` = the engine default.
    pub stability: Option<FrameCount>,
    /// Stage scope; `None` applies globally.
    pub stage: Option<StageName>,
}
