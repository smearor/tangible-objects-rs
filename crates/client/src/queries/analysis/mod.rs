//! Analysis queries - `analyzeRecording`, `analyzeSimilarity` and
//! `recordingArtifact`, built via cynic. `JSON` scalar fields select
//! as `serde_json::Value`; the response decodes into the typed
//! reports through their `Deserialize`.

pub(crate) mod api;
pub(crate) mod artifact;
pub(crate) mod recording;
pub(crate) mod similarity;
