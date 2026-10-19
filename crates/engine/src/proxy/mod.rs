//! Filter proxy (FILTER_PROXY_VIRTUAL.md): the real evdev device is grabbed
//! exclusively and its events are re-emitted through a uinput clone
//! ([`VirtualTouchscreen`]) - minus the slots that a matched tangible
//! occupies ([`Pipeline`]). The operating system sees the clone; tangible
//! contacts never reach it.

mod boundary;
mod edge_stage;
mod field_stage;
mod legacy_mode;
mod pipeline;
mod scope_changes;
mod stage;
mod stage_frame;
mod stage_metrics;
mod stages;
mod tangible_stage;
mod virtual_device;
mod zone_stage;

pub use legacy_mode::LegacyMode;
pub use pipeline::Pipeline;
pub use scope_changes::ScopeChanges;
pub use stage_metrics::StageMetrics;
pub use virtual_device::VirtualTouchscreen;
