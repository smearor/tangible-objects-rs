//! Tool topic modules - each holds one `#[tool_router(router = ..)]`
//! impl block on [`EngineMcp`](crate::server::EngineMcp); the named
//! routers merge in `crate::server::handler`.

mod analysis;
mod completion;
mod dead_zone;
mod device;
mod field_set;
mod filter;
mod gui;
mod message;
mod metrics;
mod owner;
mod pattern;
mod profiles;
mod recording;
mod spell;
mod stage;
mod tangibles;
mod training;
