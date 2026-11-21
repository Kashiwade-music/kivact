//! Role: The control, icon and interaction metrics of Astryx Neutral, in logical px (crate-private).

pub(crate) const CONTROL_XS: f64 = 24.0;
pub(crate) const CONTROL_SM: f64 = 28.0;
pub(crate) const CONTROL_MD: f64 = 32.0;
pub(crate) const CONTROL_LG: f64 = 36.0;
pub(crate) const BORDER_WIDTH: f64 = 1.0;
pub(crate) const FOCUS_RING_WIDTH: f64 = 2.0;
pub(crate) const FOCUS_RING_OFFSET: f64 = 3.0;
pub(crate) const ICON_XSM: f64 = 12.0;
pub(crate) const ICON_SM: f64 = 16.0;
pub(crate) const ICON_MD: f64 = 20.0;
pub(crate) const ICON_LG: f64 = 24.0;
pub(crate) const ICON_STROKE_RATIO: f64 = 2.0 / 24.0;
pub(crate) const BADGE_HEIGHT: f64 = 20.0;
pub(crate) const MIN_TOUCH_TARGET: f64 = 44.0;
pub(crate) const DISABLED_OPACITY: f64 = 0.5;
pub(crate) const PRESS_SCALE: f64 = 0.98;

/// The size of a presence marker on an avatar of `side`, with its ring and glyph.
pub(crate) struct StatusMetrics {
    pub(crate) side: f64,
    pub(crate) border: f64,
    pub(crate) stroke: f64,
    pub(crate) icon: f64,
}

/// The presence marker's metrics for an avatar `side` logical px wide.
pub(crate) fn status_metrics(side: f64) -> StatusMetrics {
    if side <= 36.0 {
        StatusMetrics {
            side: 10.0,
            border: 1.0,
            stroke: 1.0,
            icon: 0.0,
        }
    } else if side < 96.0 {
        StatusMetrics {
            side: 20.0,
            border: 2.0,
            stroke: 1.5,
            icon: 12.0,
        }
    } else {
        StatusMetrics {
            side: 32.0,
            border: 4.0,
            stroke: 2.0,
            icon: 18.0,
        }
    }
}
