//! Tool argument structs - one file per tool, `Deserialize` +
//! `JsonSchema` for the rmcp parameter surface.

mod analyze;
mod get_pose;
mod record_label;
mod recording_artifact;
mod register_field_set;
mod register_pattern;
mod set_completion;
mod set_filter_mode;
mod set_gui_visible;
mod show_message;
mod similarity;
mod stage;
mod start_recording;
mod train_profile;
mod unregister_field_set;
mod unregister_owner;
mod unregister_pattern;
mod update_profile;

pub use analyze::AnalyzeArgs;
pub use get_pose::GetPoseArgs;
pub use record_label::SetRecordLabelArgs;
pub use recording_artifact::RecordingArtifactArgs;
pub use register_field_set::RegisterFieldSetArgs;
pub use register_pattern::RegisterPatternArgs;
pub use set_completion::SetCompletionArgs;
pub use set_filter_mode::SetFilterModeArgs;
pub use set_gui_visible::SetGuiVisibleArgs;
pub use show_message::ShowMessageArgs;
pub use similarity::SimilarityArgs;
pub use stage::StageArgs;
pub use start_recording::StartRecordingArgs;
pub use train_profile::TrainProfileArgs;
pub use unregister_field_set::UnregisterFieldSetArgs;
pub use unregister_owner::UnregisterOwnerArgs;
pub use unregister_pattern::UnregisterPatternArgs;
pub use update_profile::UpdateProfileArgs;
