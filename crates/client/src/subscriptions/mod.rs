//! Subscription-side methods of [`crate::GraphQLClient`] - each
//! returns a decoded [`tangible_objects_model::TangibleEvent`]
//! stream. One directory per schema operation: `api.rs` holds the
//! client method, `query.rs`/`variables.rs` the cynic variables and
//! `sub.rs` the cynic fragment.

pub(crate) mod events;
pub(crate) mod fields;
pub(crate) mod tangibles;
