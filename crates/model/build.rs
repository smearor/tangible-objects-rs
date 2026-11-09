//! Rebuilds the model crate when the embedded GraphQL schema changes.
//!
//! `cynic::use_schema!` reads `../graphql/schema.graphql` at macro-expansion
//! time but does not register it as a tracked input - without this hint the
//! crate keeps stale `MutationRoot`/`QueryRoot` types after a schema regen or
//! a `git pull` that updates the file.

fn main() {
    println!("cargo:rerun-if-changed=../graphql/schema.graphql");
}
