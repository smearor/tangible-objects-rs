//! Query-side methods of [`crate::GraphQLClient`] - one file per
//! schema area; `profile_query` holds the `profiles` narrow type.

mod analysis;
mod completion;
mod device;
mod message;
mod metrics;
mod profile_query;
mod profiles;
mod recording;
mod sets;
mod tangibles;
mod touches;

pub use profile_query::ProfileQuery;
