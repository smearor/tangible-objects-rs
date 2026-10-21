//! Daemon modes: replay, live detection, training and device listing.
//!
//! `setup` holds the shared device/config loading, `control` the
//! control-command handling the live modes share, and each `run_*`
//! entry point has its own file.

mod analysis;
mod control;
mod filter;
mod interfaces;
mod live;
mod record;
mod replay;
mod setup;
mod store;
mod train;

pub use analysis::run_analyze;
pub use analysis::run_convert_device;
pub use analysis::run_export_dataset;
pub use analysis::run_similarity;
// Re-exported for the gui feature's `gtk_app` - unused without it.
#[cfg(feature = "gui")]
pub(crate) use control::drain_control_commands;
#[cfg(feature = "gui")]
pub(crate) use control::spawn_interfaces;
#[cfg(feature = "gui")]
pub(crate) use control::status_of_session;
pub use filter::run_filter;
pub use interfaces::Interfaces;
pub use live::run_live;
pub use record::run_record;
pub use replay::run_replay;
#[cfg(feature = "gui")]
pub use tangible_objects_model::DetectionConfig;
// Only the gui feature calls this through `modes::` - the modes
// themselves use `super::setup::effective_calibration` directly.
#[cfg(feature = "gui")]
pub use setup::effective_calibration;
pub use setup::list_devices;
pub use setup::load_completion;
pub use setup::load_field_sets;
pub use setup::load_patterns;
pub use setup::resolve_device;
pub use train::run_train;
