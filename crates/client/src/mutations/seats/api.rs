//! Seat session methods of [`GraphQLClient`] - `claim_seat_session`,
//! `release_seat_session`, `add_seat`, `update_seat` and
//! `remove_seat` (TANGIBLE_OBJECTS_TABLE_TOP_SEAT.md).

use smearor_commons_layout::SeatPose;
use tangible_objects_model::AddSeatResult;
use tangible_objects_model::ClaimSeatSessionResult;
use tangible_objects_model::Owner;
use tangible_objects_model::PatternId;
use tangible_objects_model::ReleaseSeatSessionResult;
use tangible_objects_model::RemoveSeatResult;
use tangible_objects_model::SeatId;
use tangible_objects_model::SeatName;
use tangible_objects_model::SeatSessionName;
use tangible_objects_model::UpdateSeatResult;

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::seats::add::mutation::AddSeat;
use crate::mutations::seats::add::result::AddSeatResultFragment;
use crate::mutations::seats::add::variables::AddSeatVariables;
use crate::mutations::seats::claim::mutation::ClaimSeatSession;
use crate::mutations::seats::claim::result::ClaimSeatSessionResultFragment;
use crate::mutations::seats::claim::variables::ClaimSeatSessionVariables;
use crate::mutations::seats::input::AddSeatInput;
use crate::mutations::seats::input::UpdateSeatInput;
use crate::mutations::seats::release::mutation::ReleaseSeatSession;
use crate::mutations::seats::release::result::ReleaseSeatSessionResultFragment;
use crate::mutations::seats::release::variables::ReleaseSeatSessionVariables;
use crate::mutations::seats::remove::mutation::RemoveSeat;
use crate::mutations::seats::remove::result::RemoveSeatResultFragment;
use crate::mutations::seats::remove::variables::RemoveSeatVariables;
use crate::mutations::seats::update::mutation::UpdateSeat;
use crate::mutations::seats::update::result::UpdateSeatResultFragment;
use crate::mutations::seats::update::variables::UpdateSeatVariables;

/// A seat mutation as the application intends it - the pieces of
/// `UpdateSeatInput` that are meaningful to callers, folded onto the
/// wire input in [`GraphQLClient::update_seat`].
#[derive(Debug, Default)]
pub struct SeatUpdate {
    /// New seat name - `None` keeps the current one.
    pub name: Option<SeatName>,
    /// Rebinds the marker profile - `None` keeps the current one.
    /// Takes precedence over [`Self::clear_marker`].
    pub marker: Option<PatternId>,
    /// Clears the marker binding so the seat turns virtual.
    pub clear_marker: bool,
    /// New pose - `None` keeps the current one.
    pub pose: Option<SeatPose>,
}

impl GraphQLClient {
    /// Claims a seat session for `owner` (`claimSeatSession`) -
    /// idempotent for the same owner, rejected with
    /// `SESSION_ALREADY_OWNED` for a foreign claim. The session name
    /// is the roster's identity on every interface.
    pub async fn claim_seat_session(
        &self,
        name: SeatSessionName,
        owner: &Owner,
    ) -> Result<ClaimSeatSessionResult, ClientError> {
        let fragment: ClaimSeatSessionResultFragment = self
            .call_mutation::<ClaimSeatSession, _, _>(
                "claimSeatSession",
                ClaimSeatSessionVariables {
                    name,
                    owner: owner.clone(),
                },
            )
            .await?;
        fragment.into_result()
    }

    /// Releases `owner`'s claim on the session
    /// (`releaseSeatSession`) - its seats are removed with it.
    pub async fn release_seat_session(
        &self,
        name: SeatSessionName,
        owner: &Owner,
    ) -> Result<ReleaseSeatSessionResult, ClientError> {
        let fragment: ReleaseSeatSessionResultFragment = self
            .call_mutation::<ReleaseSeatSession, _, _>(
                "releaseSeatSession",
                ReleaseSeatSessionVariables {
                    name,
                    owner: owner.clone(),
                },
            )
            .await?;
        fragment.into_result()
    }

    /// Adds a seat to a claimed session (`addSeat`) - `marker` set
    /// anchors the seat to that profile's tracked tangibles; `marker:
    /// None` creates a virtual seat at `pose`.
    pub async fn add_seat(
        &self,
        session: SeatSessionName,
        name: SeatName,
        marker: Option<PatternId>,
        pose: Option<SeatPose>,
        owner: &Owner,
    ) -> Result<AddSeatResult, ClientError> {
        let fragment: AddSeatResultFragment = self
            .call_mutation::<AddSeat, _, _>(
                "addSeat",
                AddSeatVariables {
                    input: AddSeatInput {
                        session,
                        name,
                        marker,
                        pose: pose.map(Into::into),
                    },
                    owner: owner.clone(),
                },
            )
            .await?;
        fragment.into_result()
    }

    /// Mutates a seat addressed by its globally unique ID
    /// (`updateSeat`) - `None` fields keep their value.
    pub async fn update_seat(
        &self,
        id: SeatId,
        update: SeatUpdate,
        owner: &Owner,
    ) -> Result<UpdateSeatResult, ClientError> {
        let input = UpdateSeatInput {
            name: update.name,
            marker: update.marker,
            clear_marker: update.clear_marker.then_some(true),
            pose: update.pose.map(Into::into),
        };
        let fragment: UpdateSeatResultFragment = self
            .call_mutation::<UpdateSeat, _, _>(
                "updateSeat",
                UpdateSeatVariables {
                    id,
                    input,
                    owner: owner.clone(),
                },
            )
            .await?;
        fragment.into_result()
    }

    /// Removes a seat from its session (`removeSeat`).
    pub async fn remove_seat(
        &self,
        id: SeatId,
        owner: &Owner,
    ) -> Result<RemoveSeatResult, ClientError> {
        let fragment: RemoveSeatResultFragment = self
            .call_mutation::<RemoveSeat, _, _>(
                "removeSeat",
                RemoveSeatVariables {
                    id,
                    owner: owner.clone(),
                },
            )
            .await?;
        fragment.into_result()
    }
}
