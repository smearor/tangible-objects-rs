//! Domain model for the tangible-objects platform.
//!
//! Contains the pure, OS-agnostic types shared by every crate: touch point
//! geometry, device calibration, geometric figure patterns, TOML profiles and
//! tracked tangible state. No I/O beyond profile file loading/saving.

mod calibration;
mod device;
mod event;
mod filter_mode;
mod millimeters;
mod milliseconds;
mod pattern;
mod pixels;
mod profile;
mod raw_axis_value;
mod recording;
mod rotation_rad;
mod set;
mod tangible;
mod touch_point;
mod units_per_mm;
mod vec2;

pub use calibration::Calibration;
pub use device::DeviceError;
pub use device::DeviceProfile;
pub use device::DeviceRegistry;
pub use event::ABS_MT_POSITION_X;
pub use event::ABS_MT_POSITION_Y;
pub use event::ABS_MT_SLOT;
pub use event::ABS_MT_TRACKING_ID;
pub use event::ABS_X;
pub use event::ABS_Y;
pub use event::BTN_TOUCH;
pub use event::EventCode;
pub use event::EventCount;
pub use event::EventType;
pub use event::NamedEventCode;
pub use event::RawEvent;
pub use event::SYN_REPORT;
pub use filter_mode::FilterMode;
pub use millimeters::Millimeters;
pub use milliseconds::Milliseconds;
pub use pattern::PatternId;
pub use pattern::PatternMatch;
pub use pattern::PatternTolerance;
pub use pattern::TangiblePattern;
pub use pixels::Pixels;
pub use profile::ProfileDir;
pub use profile::ProfileError;
pub use profile::ProfileFile;
pub use raw_axis_value::RawAxisValue;
pub use recording::CandidateCount;
pub use recording::CandidateReport;
pub use recording::FrameCount;
pub use recording::FrameNumber;
pub use recording::FrameRecord;
pub use recording::MatchScore;
pub use recording::RECORD_SCHEMA_VERSION;
pub use recording::RecordLabel;
pub use recording::RemainingFrameCount;
pub use recording::TrackedRecord;
pub use rotation_rad::RotationRad;
pub use set::SetAction;
pub use set::SetTracker;
pub use set::TangibleSet;
pub use set::TangibleSetId;
pub use set::TangibleSetState;
pub use tangible::TangibleId;
pub use tangible::TrackedTangible;
pub use touch_point::TouchPoint;
pub use touch_point::TouchPointSlot;
pub use touch_point::TrackingId;
pub use units_per_mm::UnitsPerMm;
pub use vec2::Vec2;
