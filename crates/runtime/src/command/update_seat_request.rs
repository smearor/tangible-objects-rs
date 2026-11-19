use smearor_commons_layout::SeatPose;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::RegistrationError;
use tangible_objects_model::SeatId;
use tangible_objects_model::SeatName;
use tangible_objects_model::TableTopSeat;
use tokio::sync::oneshot;

/// Payload of
/// [`ControlCommand::UpdateSeat`](crate::ControlCommand::UpdateSeat) -
/// boxed like [`crate::AddSeatRequest`].
#[derive(Debug)]
pub struct UpdateSeatRequest {
    /// Seat to mutate - addressed by its globally unique ID.
    pub seat: SeatId,
    /// Owner mutating the seat - must own its session.
    pub owner: Owner,
    /// New seat name - `None` keeps the current one.
    pub name: Option<SeatName>,
    /// `None` keeps the marker, `Some(None)` clears it,
    /// `Some(Some)` rebinds to another profile (validated like
    /// [`crate::AddSeatRequest::marker`]).
    pub marker: Option<Option<PatternId>>,
    /// New pose - `None` keeps the current one.
    pub pose: Option<SeatPose>,
    /// Receives the updated seat or the typed rejection.
    pub reply: oneshot::Sender<Result<TableTopSeat, RegistrationError>>,
}
