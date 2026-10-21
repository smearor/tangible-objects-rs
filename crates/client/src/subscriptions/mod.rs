//! Subscription-side methods of [`crate::GraphQLClient`] - each
//! returns a decoded [`tangible_objects_model::TangibleEvent`] stream;
//! `event_stream_query` holds the `eventStream` filter type.

mod event_stream_query;
mod events;
mod fields;
mod tangibles;

pub use event_stream_query::EventStreamQuery;
