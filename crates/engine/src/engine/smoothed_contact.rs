use tangible_objects_model::Millimeters;
use tangible_objects_model::SmoothingAlpha;
use tangible_objects_model::TrackingId;
use tangible_objects_model::Vec2;

/// Per-slot EMA smoothing state (TRACKING_STABILIZATION.md): the
/// tracking ID the smoothed position belongs to plus the position
/// itself. The ID guards the blend - a slot reuse by a *new* contact
/// must not inherit the old contact's smoothed position.
pub(crate) struct SmoothedContact {
    /// Tracking ID owning the smoothed position.
    pub tracking_id: TrackingId,
    /// EMA-smoothed position.
    pub pos: Vec2<Millimeters>,
}

impl SmoothedContact {
    /// Fresh state - no prior position, `pos` is kept as-is.
    pub fn new(tracking_id: TrackingId, pos: Vec2<Millimeters>) -> Self {
        Self { tracking_id, pos }
    }

    /// The smoothed position for an incoming `pos` owned by `id`:
    /// EMA-blended toward the stored position when `id` matches this
    /// state's owner, the raw `pos` on a tracking-ID mismatch (stale
    /// state - a slot reuse by a new contact must not inherit the old
    /// contact's position).
    pub fn blend(
        &self,
        alpha: SmoothingAlpha,
        id: TrackingId,
        pos: Vec2<Millimeters>,
    ) -> Vec2<Millimeters> {
        if self.tracking_id == id {
            let alpha = alpha.as_f64();
            Vec2::new(
                alpha * pos.x + (1.0 - alpha) * self.pos.x,
                alpha * pos.y + (1.0 - alpha) * self.pos.y,
            )
        } else {
            pos
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SmoothedContact;
    use tangible_objects_model::SmoothingAlpha;
    use tangible_objects_model::TrackingId;
    use tangible_objects_model::Vec2;

    /// A matching tracking ID EMA-blends toward the stored position.
    #[test]
    fn blend_ema_when_ids_match() {
        let state = SmoothedContact::new(TrackingId::new(1), Vec2::new(10.0, 20.0));
        let blended = state.blend(
            SmoothingAlpha::new(0.5),
            TrackingId::new(1),
            Vec2::new(30.0, 40.0),
        );
        assert_eq!(blended, Vec2::new(20.0, 30.0));
    }

    /// A slot reuse by a new contact must not inherit the old contact's
    /// smoothed position - the raw `pos` is returned.
    #[test]
    fn blend_returns_raw_on_id_mismatch() {
        let state = SmoothedContact::new(TrackingId::new(1), Vec2::new(10.0, 20.0));
        let blended = state.blend(
            SmoothingAlpha::new(0.5),
            TrackingId::new(2),
            Vec2::new(30.0, 40.0),
        );
        assert_eq!(blended, Vec2::new(30.0, 40.0));
    }
}
