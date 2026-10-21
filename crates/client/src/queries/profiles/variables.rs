/// Narrowings for the `profiles` query - `None` everywhere asks for
/// every profile the daemon reports. Serialized by cynic as the
/// operation's variables.
#[derive(Debug, Default, Clone, cynic::QueryVariables)]
#[cynic(
    schema_module = "tangible_objects_model::schema",
    rename_all = "camelCase"
)]
pub struct ProfileQuery {
    /// Only profiles owned by this application (`register_pattern`'s
    /// `owner`).
    pub owner: Option<String>,
    /// `true` asks only for statically loaded profiles (no runtime
    /// registrations).
    pub static_only: Option<bool>,
}
