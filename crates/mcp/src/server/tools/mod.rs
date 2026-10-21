//! Tool topic modules - each holds one `#[tool_router(router = ..)]`
//! impl block on [`EngineMcp`](crate::server::EngineMcp); the named
//! routers merge in `crate::server::handler`.

mod analysis;
mod completion;
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
mod stage;
mod tangibles;
mod training;
