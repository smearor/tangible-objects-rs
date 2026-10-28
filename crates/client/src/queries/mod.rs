//! Query-side methods of [`crate::GraphQLClient`] - one directory per
//! schema area: `mod.rs` holds the client methods, the cynic
//! variables and fragments live in one file per type.

pub(crate) mod analysis;
pub(crate) mod completion;
pub(crate) mod device;
pub(crate) mod field_sets;
pub(crate) mod message;
pub(crate) mod metrics;
pub(crate) mod profiles;
pub(crate) mod recording;
pub(crate) mod sets;
pub(crate) mod tangibles;
pub(crate) mod touches;
