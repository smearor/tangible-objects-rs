//! Cross-device `.evtest` conversion: replay a recording made on device A
//! and re-emit it as raw events of device B.

mod converter;
mod stats;

pub use converter::DeviceConverter;
pub use stats::ConvertStats;
