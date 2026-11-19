//! Mutation-side methods of [`crate::GraphQLClient`] - one directory
//! per schema area: `mod.rs` holds the client methods, the cynic
//! input objects, variables and fragments live in one file per type.

pub(crate) mod completion;
pub(crate) mod dead_zone;
pub(crate) mod field_set;
pub(crate) mod filter;
pub(crate) mod gui;
pub(crate) mod message;
pub(crate) mod metrics;
pub(crate) mod pattern;
pub(crate) mod recording;
pub(crate) mod seats;
pub(crate) mod spell;
pub(crate) mod stage;
pub(crate) mod training;
