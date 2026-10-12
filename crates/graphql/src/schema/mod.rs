mod mutation;
mod query;
mod subscription;

pub use mutation::MutationRoot;
pub use query::QueryRoot;
pub use subscription::SubscriptionRoot;

/// Engine GraphQL schema (query + mutation + subscription).
pub type EngineSchema = async_graphql::Schema<QueryRoot, MutationRoot, SubscriptionRoot>;
