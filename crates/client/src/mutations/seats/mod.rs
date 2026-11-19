//! Seat session lifecycle mutations - `claimSeatSession`,
//! `releaseSeatSession`, `addSeat`, `updateSeat` and `removeSeat`,
//! built via cynic.

pub(crate) mod add;
pub(crate) mod api;
pub(crate) mod claim;
pub(crate) mod input;
pub(crate) mod release;
pub(crate) mod remove;
pub(crate) mod update;
