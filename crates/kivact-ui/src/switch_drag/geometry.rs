//! Role: Resolve switch thumb travel and rubber-band geometry.
use crate::values::ControlSize;
use kivact_core::view::{ReadingDirection, use_direction};
#[derive(Clone, Copy)]
pub(super) struct Geometry {
    pub(super) width: f64,
    pub(super) height: f64,
    pub(super) off_side: f64,
    pub(super) on_side: f64,
    pub(super) off_inset: f64,
    pub(super) sign: f64,
}

impl Geometry {
    pub(super) fn new(size: ControlSize) -> Self {
        let sign = if use_direction().get() == ReadingDirection::Rtl {
            -1.0
        } else {
            1.0
        };
        if matches!(size, ControlSize::ExtraSmall | ControlSize::Small) {
            Self {
                width: 32.0,
                height: 20.0,
                off_side: 14.0,
                on_side: 16.0,
                off_inset: 3.0,
                sign,
            }
        } else {
            Self {
                width: 40.0,
                height: 24.0,
                off_side: 16.0,
                on_side: 20.0,
                off_inset: 4.0,
                sign,
            }
        }
    }

    pub(super) fn travel(self) -> f64 {
        self.width - 2.0 - self.on_side / 2.0 - self.off_inset - self.off_side / 2.0
    }

    pub(super) fn band(self, raw: f64, on: bool) -> f64 {
        let end = self.travel();
        let inset = if on { 2.0 } else { self.off_inset };
        let dimension = (12.0 * self.height / 28.0).min(inset / 2.0);
        let clamped = raw.clamp(0.0, end);
        let excess = raw - clamped;
        clamped
            + excess.signum() * (1.0 - 1.0 / (excess.abs() * 0.55 / dimension + 1.0)) * dimension
    }
}
