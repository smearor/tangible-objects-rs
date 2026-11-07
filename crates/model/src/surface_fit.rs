use crate::DisplayRotation;
use crate::Millimeters;
use crate::Pixels;
use crate::PixelsPerMm;
use crate::Vec2;

/// Composed mm↔px mapping of a surface onto a display area.
///
/// Bundles the three pieces every renderer or hit-test needs so they
/// cannot drift apart: the display `rotation` (the evdev stream always
/// speaks the panel's native frame), the uniform `scale`
/// ([`PixelsPerMm`], letterboxed so figures keep their physical
/// aspect) and the `offset` centering the fitted surface inside the
/// area. Built by `Calibration::surface_fit`; `map` is the render
/// direction, `unmap` the hit-test direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceFit {
    /// Uniform scale applied after rotation - same factor on both
    /// axes (uniform fit, no aspect distortion).
    pub scale: PixelsPerMm,
    /// Letterbox offset of the rotated surface inside the display
    /// area, in pixels.
    pub offset: Vec2<Pixels>,
    /// Quarter turn applied to mm positions before scaling.
    pub rotation: DisplayRotation,
    /// Rotated surface size in millimeters - `(h, w)` when the quarter
    /// turn swaps axes.
    pub size: Vec2<Millimeters>,
}

impl SurfaceFit {
    /// Builds the fit for a `w_mm` × `h_mm` surface inside a
    /// `w_px` × `h_px` display area under `rotation`.
    pub fn new(
        w_mm: Millimeters,
        h_mm: Millimeters,
        w_px: Pixels,
        h_px: Pixels,
        rotation: DisplayRotation,
    ) -> Self {
        let (w_disp, h_disp) = if rotation.swaps_axes() {
            (h_mm, w_mm)
        } else {
            (w_mm, h_mm)
        };
        let scale = if w_disp.as_f64() > 0.0 && h_disp.as_f64() > 0.0 {
            (w_px.as_f64() / w_disp.as_f64()).min(h_px.as_f64() / h_disp.as_f64())
        } else {
            0.0
        };
        let scaled = Vec2::<Pixels>::new(w_disp.as_f64() * scale, h_disp.as_f64() * scale);
        Self {
            scale: PixelsPerMm::new(scale),
            offset: (Vec2::from_px(w_px, h_px) - scaled) / 2.0,
            rotation,
            size: Vec2::new(w_disp.as_f64(), h_disp.as_f64()),
        }
    }

    /// Maps a millimeter position (panel-native frame) onto display
    /// pixels: rotate → uniform scale → letterbox offset.
    pub fn map(&self, pos: Vec2<Millimeters>) -> Vec2<Pixels> {
        let rotated = self.rotation.apply(
            pos,
            if self.rotation.swaps_axes() {
                Millimeters::new(self.size.y)
            } else {
                Millimeters::new(self.size.x)
            },
            if self.rotation.swaps_axes() {
                Millimeters::new(self.size.x)
            } else {
                Millimeters::new(self.size.y)
            },
        );
        let scaled = self.scale.scale_vec(rotated);
        self.offset + scaled
    }

    /// Maps a display pixel back onto surface millimeters (panel-native
    /// frame) - the hit-testing direction: inverse offset → unscale →
    /// inverse rotation.
    pub fn unmap(&self, px: Vec2<Pixels>) -> Vec2<Millimeters> {
        let scaled = self.scale.unscale_vec(px - self.offset);
        self.rotation.inverse().apply(
            scaled,
            Millimeters::new(self.size.x),
            Millimeters::new(self.size.y),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::SurfaceFit;
    use crate::DisplayRotation;
    use crate::Millimeters;
    use crate::Pixels;
    use crate::Vec2;

    /// 400×300 mm surface on a 1000×1000 px display.
    fn fit(rotation: DisplayRotation) -> SurfaceFit {
        SurfaceFit::new(
            Millimeters::new(400.0),
            Millimeters::new(300.0),
            Pixels::new(1000.0),
            Pixels::new(1000.0),
            rotation,
        )
    }

    #[test]
    fn map_scales_and_letterboxes() {
        let fit = fit(DisplayRotation::Deg0);
        // min(2.5, 3.33) = 2.5 px/mm, vertical letterbox 125 px
        assert!((fit.scale.as_f64() - 2.5).abs() < 1e-9);
        let mapped = fit.map(Vec2::<Millimeters>::new(0.0, 0.0));
        assert!((mapped.y - 125.0).abs() < 1e-9);
        assert!((fit.map(Vec2::<Millimeters>::new(400.0, 300.0)).x - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn unmap_inverts_map_for_every_rotation() {
        let point = Vec2::<Millimeters>::new(111.0, 222.0);
        for rotation in [
            DisplayRotation::Deg0,
            DisplayRotation::Deg90,
            DisplayRotation::Deg180,
            DisplayRotation::Deg270,
        ] {
            let fit = fit(rotation);
            let back = fit.unmap(fit.map(point));
            assert!(
                (back.x - point.x).abs() < 1e-6 && (back.y - point.y).abs() < 1e-6,
                "{rotation:?} round-trip: ({},{}) vs ({},{})",
                point.x,
                point.y,
                back.x,
                back.y
            );
        }
    }
}
