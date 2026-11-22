//! `RegisterHandProfileInput` - the `registerHandProfile` input
//! object, a wire DTO of [`HandProfile`] (SPELLS.md's per-user
//! spells).

use async_graphql::Error;
use async_graphql::Result;
use smearor_commons_geometry::Millimeters;
use smearor_commons_geometry::Vec2;
use tangible_objects_model::HandName;
use tangible_objects_model::HandProfile;

/// The handprint tolerance default in millimeters when a
/// registration leaves it unset.
const DEFAULT_HAND_TOLERANCE_MM: f64 = 10.0;

/// Input for `registerHandProfile`: a named five-contact
/// constellation a `five-finger-tap` arming can bind spells to.
#[derive(async_graphql::InputObject)]
pub(crate) struct RegisterHandProfileInput {
    /// Hand name (identity for unregister and the runtime store -
    /// e.g. the user's name).
    name: HandName,
    /// Max per-distance deviation in millimeters; `null` = 10 mm.
    tolerance: Option<Millimeters>,
    /// The recorded contact constellation - flat `[x0, y0, x1, y1,
    /// ...]` pairs in millimeters (same shape as
    /// `registerPattern.points`).
    points: Vec<Millimeters>,
}

impl TryFrom<&RegisterHandProfileInput> for HandProfile {
    type Error = Error;

    /// Projects the wire input into the domain handprint - validates
    /// the point arity.
    fn try_from(input: &RegisterHandProfileInput) -> Result<Self> {
        if input.points.len() < 2 || !input.points.len().is_multiple_of(2) {
            return Err(Error::new("hand point list must be flat [x, y] pairs"));
        }
        let (pairs, _) = input.points.as_chunks::<2>();
        let points: Vec<Vec2<Millimeters>> = pairs
            .iter()
            .map(|pair| Vec2::from_mm(pair[0], pair[1]))
            .collect();
        Ok(Self::from_points(
            input.name.clone(),
            &points,
            input
                .tolerance
                .unwrap_or(Millimeters::new(DEFAULT_HAND_TOLERANCE_MM)),
        ))
    }
}
