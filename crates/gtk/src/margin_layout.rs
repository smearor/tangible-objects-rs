//! `MarginLayout` - a [`LayoutStrategy`] decorator that insets the
//! arrangement bounds by a uniform margin before delegating to an
//! inner strategy, restoring the edge breathing room the former
//! hand-rolled default arrangement kept.

use smearor_commons_geometry::Pixels;
use smearor_commons_geometry::Rect;
use smearor_commons_layout::FlowLayout;
use smearor_commons_layout::LayoutError;
use smearor_commons_layout::LayoutItem;
use smearor_commons_layout::LayoutStrategy;
use smearor_commons_layout::Placement;

/// Insets the arrangement bounds by `margin` on every side, then
/// defers the packing to `inner`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct MarginLayout {
    /// The wrapped packing strategy.
    inner: FlowLayout<Pixels>,
    /// Uniform margin around the arrangement bounds.
    margin: Pixels,
}

impl MarginLayout {
    /// A margin-inset layout around `inner`.
    pub(crate) fn new(inner: FlowLayout<Pixels>, margin: Pixels) -> Self {
        Self { inner, margin }
    }
}

impl LayoutStrategy<Pixels> for MarginLayout {
    fn arrange(
        &self,
        bounds: Rect<Pixels>,
        items: &[LayoutItem<Pixels>],
    ) -> Result<Vec<Placement<Pixels>>, LayoutError> {
        self.inner.arrange(bounds.inset(self.margin), items)
    }
}
