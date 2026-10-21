use crate::EventCount;
use crate::FrameCount;
use crate::RecordLabel;
#[cfg(feature = "cynic")]
use crate::schema;
use serde::Deserialize;
use serde::Serialize;

/// Live status of the daemon's recording session.
///
/// Published inside [`crate::EngineSnapshot`] - interfaces read it via the
/// `recording_status` query; `None` in the snapshot means no session is or
/// was active.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
#[cfg_attr(feature = "cynic", derive(cynic::QueryFragment))]
#[cfg_attr(
    feature = "cynic",
    cynic(
        schema_path = "../graphql/schema.graphql",
        graphql_type = "RecordingStatus",
        no_deserialize,
        rename_all = "camelCase"
    )
)]
pub struct RecordingStatus {
    /// Whether a session is currently recording.
    pub active: bool,
    /// Current ground-truth label, if any.
    pub label: Option<RecordLabel>,
    /// Frames written so far (non-empty frames only).
    pub recorded_frames: FrameCount,
    /// Frames dropped on a full frame-sink buffer.
    pub dropped_frames: FrameCount,
    /// Raw events dropped on a full raw-sink buffer.
    pub dropped_events: EventCount,
}
