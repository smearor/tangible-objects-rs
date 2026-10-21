//! Prints the engine schema's SDL to stdout - the wire contract for
//! codegen, docs and client-side document validation:
//!
//! ```sh
//! cargo run -p tangible-objects-graphql --bin export-schema > schema.graphql
//! ```

use tangible_objects_graphql::EngineSchema;
use tangible_objects_graphql::MutationRoot;
use tangible_objects_graphql::QueryRoot;
use tangible_objects_graphql::SubscriptionRoot;

/// Entry point - builds the schema (no `EngineHandle` needed, only
/// resolvers consume it) and prints its SDL.
fn main() {
    let schema = EngineSchema::build(QueryRoot, MutationRoot, SubscriptionRoot).finish();
    println!("{}", schema.sdl());
}
