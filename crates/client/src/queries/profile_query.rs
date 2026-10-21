use serde::Serialize;

/// Narrowings for the `profiles` query - `None` everywhere asks for
/// every profile the daemon reports.
#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileQuery {
    /// Only profiles owned by this application (`register_pattern`'s
    /// `owner`).
    pub owner: Option<String>,
    /// `true` asks only for statically loaded profiles (no runtime
    /// registrations).
    pub static_only: Option<bool>,
}
