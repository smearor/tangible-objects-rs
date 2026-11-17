//! Pattern lifecycle mutations - `registerPattern`, `updateProfile`,
//! `unregisterPattern` and `unregisterOwner`, built via cynic.

pub(crate) mod api;
pub(crate) mod claim;
pub(crate) mod input;
pub(crate) mod register;
pub(crate) mod release;
pub(crate) mod unregister;
pub(crate) mod unregister_owner;
pub(crate) mod update;
