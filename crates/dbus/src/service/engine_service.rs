//! `EngineService` - the D-Bus service object served at
//! `/org/tangibleobjects/Engine`; the `#[interface]` impl lives in
//! `crate::service::interface`, method bodies in `crate::service::api`.

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

use tangible_objects_model::Owner;
use tangible_objects_runtime::EngineHandle;
use zbus::message::Header;
use zbus::names::UniqueName;

/// Maps a caller's unique bus name to the [`Owner`]s it registered
/// under - drives automatic cleanup when the name vanishes.
pub type OwnerSessions = Arc<Mutex<HashMap<String, HashSet<Owner>>>>;

/// D-Bus service object served at `/org/tangibleobjects/Engine`.
///
/// Methods read the latest snapshot or forward control commands to the
/// device loop; signals are emitted via
/// [`emit_signal`](crate::service::signals::emit_signal).
pub struct EngineService {
    /// Runtime handle shared by all interface methods.
    pub(crate) handle: EngineHandle,
    /// Owner sessions for disconnect cleanup (`NameOwnerChanged`).
    pub(crate) owner_sessions: OwnerSessions,
}

impl EngineService {
    /// Creates the service for the given runtime handle and the shared
    /// owner-session map watched by [`serve`](crate::serve).
    pub fn new(handle: EngineHandle, owner_sessions: OwnerSessions) -> Self {
        Self {
            handle,
            owner_sessions,
        }
    }

    /// Resolves the effective owner for a register call: the explicit
    /// `owner`, else the caller's unique bus name. Records the
    /// owner under the sender for disconnect cleanup.
    pub(crate) fn session_owner(&self, owner: Owner, header: &Header<'_>) -> Owner {
        let sender = header
            .sender()
            .map(UniqueName::as_str)
            .unwrap_or_default()
            .to_string();
        let effective = if owner.as_str().is_empty() {
            Owner::new(sender.clone())
        } else {
            owner
        };
        if !sender.is_empty()
            && let Ok(mut sessions) = self.owner_sessions.lock()
        {
            sessions
                .entry(sender)
                .or_default()
                .insert(effective.clone());
        }
        effective
    }
}
