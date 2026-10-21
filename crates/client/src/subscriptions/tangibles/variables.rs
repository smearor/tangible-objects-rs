//! `TangibleEventsVariables` - the variables of `tangibleEvents`.

use cynic::QueryVariables;
use tangible_objects_model::PatternId;

/// Variables of `tangibleEvents` - `patternId` narrows the stream to
/// one figure, `None` streams all.
#[derive(QueryVariables)]
#[cynic(schema_module = "tangible_objects_model::schema")]
pub(crate) struct TangibleEventsVariables {
    /// Optional pattern filter.
    pub pattern_id: Option<PatternId>,
}
