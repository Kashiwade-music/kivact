//! Role: Plot area and axis mapping between data values and pixels.

use super::axis::AxisScale;
use super::model::{PlotModel, PlotView};
use kivact::geometry::{Point, Rect, Size};

/// The space the axes and the title take around the plot, in logical px.
const LEFT: f64 = 64.0;
const RIGHT: f64 = 16.0;
const BOTTOM: f64 = 40.0;
const TOP_WITH_TITLE: f64 = 24.0;
const TOP: f64 = 8.0;

/// The plot area of a canvas of `size` for `model`: the rectangle the grid,
/// the series and the readouts are drawn in. Its width or height is not
/// positive when the canvas leaves no room.
pub(super) fn plot_area(size: Size, model: &PlotModel) -> Rect {
    let top = if model.title.is_empty() {
        TOP
    } else {
        TOP_WITH_TITLE
    };
    Rect::new(
        LEFT,
        top,
        size.width - LEFT - RIGHT,
        size.height - top - BOTTOM,
    )
}

/// Whether `at` lies inside `area`, edges included.
pub(super) fn within(area: Rect, at: Point) -> bool {
    at.x >= area.x && at.x <= area.right() && at.y >= area.y && at.y <= area.bottom()
}

/// The axis transform: the identity, or log10.
pub(super) fn transform(scale: AxisScale, value: f64) -> f64 {
    match scale {
        AxisScale::Linear => value,
        AxisScale::Log10 => value.log10(),
    }
}

/// The inverse of [`transform`].
pub(super) fn inverse(scale: AxisScale, value: f64) -> f64 {
    match scale {
        AxisScale::Linear => value,
        AxisScale::Log10 => 10f64.powf(value),
    }
}

/// The maps of a view over `area`: x from left to right, y from bottom to top.
pub(super) fn maps(model: &PlotModel, view: PlotView, area: Rect) -> (Map, Map) {
    let x = Map::new(model.x.scale, view.x_min, view.x_max, area.x, area.width);
    let y = Map::new(
        model.y.scale,
        view.y_min,
        view.y_max,
        area.bottom(),
        -area.height,
    );
    (x, y)
}

/// A visible range `lo..hi` of one axis, mapped onto `start..start + span`
/// pixels. A negative span runs the axis upward (the y axis).
#[derive(Clone, Copy, Debug)]
pub(super) struct Map {
    scale: AxisScale,
    lo: f64,
    hi: f64,
    start: f64,
    span: f64,
}

impl Map {
    /// The map of `lo..hi` onto pixels from `start` by `span`.
    pub(super) fn new(scale: AxisScale, lo: f64, hi: f64, start: f64, span: f64) -> Self {
        Self {
            scale,
            lo,
            hi,
            start,
            span,
        }
    }

    /// The transformed ends of the visible range.
    fn ends(&self) -> (f64, f64) {
        (
            transform(self.scale, self.lo),
            transform(self.scale, self.hi),
        )
    }

    /// The pixel of `value`. NaN and values a log axis cannot show give
    /// `None`. Infinities sit one pixel beyond the visible edge on their side
    /// (`-inf` before the start, `+inf` beyond the end); a log axis draws no
    /// `-inf`.
    pub(super) fn px(&self, value: f64) -> Option<f64> {
        if value.is_nan() {
            return None;
        }
        let direction = self.span.signum();
        if value == f64::INFINITY {
            return Some(self.start + self.span + direction);
        }
        if value == f64::NEG_INFINITY {
            return match self.scale {
                AxisScale::Linear => Some(self.start - direction),
                AxisScale::Log10 => None,
            };
        }
        if self.scale == AxisScale::Log10 && value <= 0.0 {
            return None;
        }
        Some(self.start + self.fraction(value) * self.span)
    }

    /// The value at pixel `px`.
    pub(super) fn value(&self, px: f64) -> f64 {
        let (a, b) = self.ends();
        inverse(self.scale, a + (px - self.start) / self.span * (b - a))
    }

    /// The fraction of the range that `value` lies at (outside 0..1 beyond
    /// the ends). A range of zero width gives one half.
    fn fraction(&self, value: f64) -> f64 {
        let (a, b) = self.ends();
        if b == a {
            0.5
        } else {
            (transform(self.scale, value) - a) / (b - a)
        }
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests for the plot area and axis mapping.

    use super::super::axis::PlotAxis;
    use super::*;

    fn model(title: &str) -> PlotModel {
        PlotModel::new(PlotAxis::new("x"), PlotAxis::new("y")).title(title)
    }

    #[test]
    fn values_map_to_pixels_and_back() {
        let map = Map::new(AxisScale::Log10, 1.0, 100.0, 10.0, 200.0);
        assert!((map.px(10.0).unwrap_or(f64::NAN) - 110.0).abs() < 1e-9);
        assert!((map.value(110.0) - 10.0).abs() < 1e-9);
        assert_eq!(map.px(0.0), None);
        assert_eq!(map.px(f64::NAN), None);
        let up = Map::new(AxisScale::Linear, 0.0, 1.0, 100.0, -100.0);
        assert!(up.px(1.0).unwrap_or(f64::NAN).abs() < 1e-9);
        assert_eq!(up.px(f64::NEG_INFINITY), Some(101.0));
        assert_eq!(up.px(f64::INFINITY), Some(-1.0));
    }

    #[test]
    fn a_plot_area_leaves_room_for_the_axes_and_the_title() {
        let size = Size::new(400.0, 240.0);
        let plain = plot_area(size, &model(""));
        assert_eq!(plain, Rect::new(64.0, 8.0, 320.0, 192.0));
        let titled = plot_area(size, &model("Response"));
        assert_eq!(titled.y, 24.0);
        assert!(within(plain, Point::new(64.0, 200.0)));
        assert!(!within(plain, Point::new(63.9, 200.0)));
    }
}
