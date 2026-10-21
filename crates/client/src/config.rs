//! Client-side configuration file -
//! `~/.config/tangible-objects/client.toml`.

use std::path::PathBuf;

use serde::Deserialize;

/// Contents of the client config file - one shared file for `cli` and
/// `tui` (the daemon's own config lives separately).
#[derive(Debug, Default, Deserialize)]
pub struct ClientConfig {
    /// Daemon endpoint (`http://host:port/graphql`).
    pub endpoint: Option<String>,
}

impl ClientConfig {
    /// `~/.config/tangible-objects/client.toml` - `XDG_CONFIG_HOME`
    /// aware; `None` when no config base resolves.
    pub fn config_path() -> Option<PathBuf> {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
        Some(base.join("tangible-objects").join("client.toml"))
    }

    /// Loads the config from [`ClientConfig::config_path`]; `None` when
    /// the file is absent or unreadable.
    pub fn load() -> Option<Self> {
        let path = Self::config_path()?;
        let text = std::fs::read_to_string(path).ok()?;
        toml::from_str(&text).ok()
    }
}
