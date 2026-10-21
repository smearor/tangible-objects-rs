//! The daemon endpoint as a NewType - the `--endpoint` flag,
//! `TANGIBLE_ENDPOINT` env and `client.toml` resolve into it.

use crate::ClientConfig;
use std::fmt::Display;
use std::fmt::Formatter;

/// The daemon's GraphQL endpoint URL (e.g. `http://host:port/graphql`).
///
/// Resolution order for [`Endpoint::resolve`]: explicit flag/env value
/// wins, else the `endpoint` key of `client.toml`, else
/// [`Endpoint::DEFAULT`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint(String);

impl Endpoint {
    /// Fallback endpoint when neither flag, env nor config file set one.
    pub const DEFAULT: &'static str = "http://127.0.0.1:8080/graphql";

    /// Wraps an endpoint URL - no validation beyond non-emptiness; the
    /// transport surfaces connectivity errors at call time.
    pub fn new(url: impl Into<String>) -> Self {
        Self(url.into())
    }

    /// Resolves the effective endpoint: `explicit` (flag/`env`) wins,
    /// then `client.toml`'s `endpoint`, then [`Endpoint::DEFAULT`].
    pub fn resolve(explicit: Option<String>) -> Self {
        explicit
            .or_else(|| ClientConfig::load().and_then(|c| c.endpoint))
            .map(Self::new)
            .unwrap_or_else(|| Self::new(Self::DEFAULT))
    }

    /// The endpoint URL.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The WebSocket transport URL derived from the endpoint -
    /// `http(s)://` becomes `ws(s)://` for subscription connections.
    pub fn ws_url(&self) -> String {
        self.0
            .replacen("http://", "ws://", 1)
            .replacen("https://", "wss://", 1)
    }
}

impl Display for Endpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for Endpoint {
    fn from(url: &str) -> Self {
        Self::new(url)
    }
}

impl From<String> for Endpoint {
    fn from(url: String) -> Self {
        Self::new(url)
    }
}
