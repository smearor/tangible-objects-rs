use smearor_commons_layout::SeatPose;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::RegistrationError;
use tangible_objects_model::SeatName;
use tangible_objects_model::SeatSessionName;
use tangible_objects_model::TableTopSeat;
use tokio::sync::oneshot;

/// Payload of [`ControlCommand::AddSeat`](crate::ControlCommand::AddSeat) -
/// boxed so the command enum stays small enough for the
/// `TrySendError<ControlCommand>` reply type.
#[derive(Debug)]
pub struct AddSeatRequest {
    /// Session receiving the seat.
    pub session: SeatSessionName,
    /// Owner adding the seat - must own the session.
    pub owner: Owner,
    /// Seat name - unique within the session.
    pub name: SeatName,
    /// Marker profile anchoring the seat (`None` = virtual seat).
    pub marker: Option<PatternId>,
    /// Initial pose - the applied pose for virtual seats, the
    /// placeholder until the marker binds for marker-backed seats.
    pub pose: Option<SeatPose>,
    /// Receives the added seat or the typed rejection.
    pub reply: oneshot::Sender<Result<TableTopSeat, RegistrationError>>,
}
