//! Role: Zoom, pan, key and readout arithmetic of a plot, free of signals.

use super::axis::AxisScale;
use super::heatmap::{Grid, cell};
use super::model::{PlotModel, PlotRange, PlotView};
use super::scale::{Map, inverse, maps, transform, within};
use super::series::{PlotSeries, PlotSeriesKind};
use kivact::geometry::{Point, Rect};
use kivact::view::Key;

/// The zoom factor of one keyboard step; its reciprocal zooms the other way.
const KEY_ZOOM: f64 = 1.2;
/// The share of the visible span that one arrow key moves.
const KEY_PAN: f64 = 0.1;
/// The wheel's zoom rate: one wheel step of `delta.y` zooms by `exp(delta.y × rate)`.
const WHEEL_RATE: f64 = 0.001;

/// The range `lo..hi` zoomed by `factor` about the fraction `anchor` of the
/// range (0 at `lo`, 1 at `hi`), in the axis's scale. The value under the
/// anchor stays where it is; a factor above one widens the range.
pub(super) fn zoom_range(
    lo: f64,
    hi: f64,
    scale: AxisScale,
    anchor: f64,
    factor: f64,
) -> (f64, f64) {
    let (a, b) = (transform(scale, lo), transform(scale, hi));
    let span = (b - a) * factor;
    if !span.is_finite() || span <= 0.0 {
        return (lo, hi);
    }
    let anchor_value = a + anchor * (b - a);
    let start = anchor_value - anchor * span;
    (inverse(scale, start), inverse(scale, start + span))
}

/// The range `lo..hi` moved by `shift` of its own span (in the axis's scale).
fn pan_range(lo: f64, hi: f64, scale: AxisScale, shift: f64) -> (f64, f64) {
    let (a, b) = (transform(scale, lo), transform(scale, hi));
    let by = (b - a) * shift;
    (inverse(scale, a + by), inverse(scale, b + by))
}

/// The width or height of `area`, at least one px so a division stays finite.
fn span_px(length: f64) -> f64 {
    length.max(1.0)
}

/// The view with its x span moved by `fx` and its y span by `fy`, both as
/// fractions of the span; a positive shift shows larger values.
pub(super) fn shift(model: &PlotModel, view: PlotView, fx: f64, fy: f64) -> PlotView {
    let (x_min, x_max) = pan_range(view.x_min, view.x_max, model.x.scale, fx);
    let (y_min, y_max) = pan_range(view.y_min, view.y_max, model.y.scale, fy);
    PlotView {
        x_min,
        x_max,
        y_min,
        y_max,
    }
}

/// The view after a drag of `delta` pixels from its start, with the view at
/// the press as `view`. The content follows the pointer: a drag to the right
/// shows smaller x, and a drag down shows larger y.
pub(super) fn drag_view(model: &PlotModel, view: PlotView, area: Rect, delta: Point) -> PlotView {
    shift(
        model,
        view,
        -delta.x / span_px(area.width),
        delta.y / span_px(area.height),
    )
}

/// The view zoomed by `factor` about the pointer `at`: on the y axis when
/// `vertical`, else on the x axis.
pub(super) fn zoom_about(
    model: &PlotModel,
    view: PlotView,
    area: Rect,
    at: Point,
    vertical: bool,
    factor: f64,
) -> PlotView {
    if vertical {
        let anchor = fraction(at.y, area.bottom(), area.y);
        let (y_min, y_max) = zoom_range(view.y_min, view.y_max, model.y.scale, anchor, factor);
        PlotView {
            y_min,
            y_max,
            ..view
        }
    } else {
        let anchor = fraction(at.x, area.x, area.right());
        let (x_min, x_max) = zoom_range(view.x_min, view.x_max, model.x.scale, anchor, factor);
        PlotView {
            x_min,
            x_max,
            ..view
        }
    }
}

/// Where `p` lies between `from` (0) and `to` (1), clamped.
fn fraction(p: f64, from: f64, to: f64) -> f64 {
    if to == from {
        0.5
    } else {
        ((p - from) / (to - from)).clamp(0.0, 1.0)
    }
}

/// The view after one wheel event at `at`: a vertical delta zooms about the
/// pointer (on the y axis with Ctrl), and a horizontal delta pans x, a
/// positive one showing larger x.
pub(super) fn wheel_view(
    model: &PlotModel,
    view: PlotView,
    area: Rect,
    at: Point,
    delta: Point,
    ctrl: bool,
) -> PlotView {
    let mut next = view;
    if delta.y != 0.0 {
        let factor = (delta.y * WHEEL_RATE).exp();
        next = zoom_about(model, next, area, at, ctrl, factor);
    }
    if delta.x != 0.0 {
        next = shift(model, next, delta.x / span_px(area.width), 0.0);
    }
    next
}

/// One keyboard operation on the view. Each one is one edit.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Nudge {
    /// Zoom the x axis by the factor about the middle of the view.
    ZoomX(f64),
    /// Zoom the y axis by the factor about the middle of the view.
    ZoomY(f64),
    /// Move the x span by the fraction of its width.
    PanX(f64),
    /// Move the y span by the fraction of its height.
    PanY(f64),
    /// Return to the automatic range.
    Fit,
}

impl Nudge {
    /// The operation of `key`, or `None` for a key a plot does not use: `+`
    /// and `-` zoom x, PageUp and PageDown zoom y, the arrows pan, and `0`
    /// and Home fit the data.
    pub(super) fn of_key(key: Key) -> Option<Self> {
        match key {
            Key::Char('+') => Some(Nudge::ZoomX(1.0 / KEY_ZOOM)),
            Key::Char('-') => Some(Nudge::ZoomX(KEY_ZOOM)),
            Key::PageUp => Some(Nudge::ZoomY(1.0 / KEY_ZOOM)),
            Key::PageDown => Some(Nudge::ZoomY(KEY_ZOOM)),
            Key::ArrowLeft => Some(Nudge::PanX(-KEY_PAN)),
            Key::ArrowRight => Some(Nudge::PanX(KEY_PAN)),
            Key::ArrowDown => Some(Nudge::PanY(-KEY_PAN)),
            Key::ArrowUp => Some(Nudge::PanY(KEY_PAN)),
            Key::Char('0') | Key::Home => Some(Nudge::Fit),
            _ => None,
        }
    }

    /// The range this operation leaves from `view` of `model`.
    pub(super) fn apply(self, model: &PlotModel, view: PlotView) -> PlotRange {
        match self {
            Nudge::ZoomX(factor) => {
                let (x_min, x_max) = zoom_range(view.x_min, view.x_max, model.x.scale, 0.5, factor);
                PlotRange::Manual(PlotView {
                    x_min,
                    x_max,
                    ..view
                })
            }
            Nudge::ZoomY(factor) => {
                let (y_min, y_max) = zoom_range(view.y_min, view.y_max, model.y.scale, 0.5, factor);
                PlotRange::Manual(PlotView {
                    y_min,
                    y_max,
                    ..view
                })
            }
            Nudge::PanX(fraction) => PlotRange::Manual(shift(model, view, fraction, 0.0)),
            Nudge::PanY(fraction) => PlotRange::Manual(shift(model, view, 0.0, fraction)),
            Nudge::Fit => PlotRange::Auto,
        }
    }
}

/// What the pointer reads: the nearest point of a series and its values.
#[derive(Clone, PartialEq, Debug)]
pub(super) struct Readout {
    /// The series name.
    pub(super) name: String,
    /// The data x.
    pub(super) x: f64,
    /// The data y.
    pub(super) y: f64,
    /// The cell value, for a heatmap.
    pub(super) z: Option<f64>,
    /// The point in plot pixels.
    pub(super) at: Point,
}

/// The maps and the pointer of one readout.
struct Probe {
    x: Map,
    y: Map,
    area: Rect,
    at: Point,
    pointer_x: f64,
}

/// The readout at the pointer `at`. Over a heatmap cell it reads the cell.
/// Otherwise it reads the point nearest to the pointer among the line, point
/// and stem series; a line is searched by binary search for its nearest x,
/// and only when its x values ascend. `None` when there is nothing to read.
pub(super) fn readout(model: &PlotModel, view: PlotView, area: Rect, at: Point) -> Option<Readout> {
    let (x, y) = maps(model, view, area);
    let probe = Probe {
        x,
        y,
        area,
        at,
        pointer_x: x.value(at.x),
    };
    let mut nearest: Option<(f64, Readout)> = None;
    let mut cell_read: Option<Readout> = None;
    for series in &model.series {
        match &series.kind {
            PlotSeriesKind::Heatmap { .. } => {
                if cell_read.is_none() {
                    cell_read = heatmap_read(&probe, series);
                }
            }
            _ => point_read(&probe, series, &mut nearest),
        }
    }
    cell_read.or_else(|| nearest.map(|(_, read)| read))
}

/// The readout of the heatmap cell under the pointer, if it is inside the plot.
fn heatmap_read(probe: &Probe, series: &PlotSeries) -> Option<Readout> {
    let PlotSeriesKind::Heatmap {
        x_edges,
        y_edges,
        z,
        ..
    } = &series.kind
    else {
        return None;
    };
    if !within(probe.area, probe.at) {
        return None;
    }
    let grid = Grid::of(x_edges.as_slice(), y_edges.as_slice(), z.as_slice())?;
    let (xv, yv) = (probe.pointer_x, probe.y.value(probe.at.y));
    let c = cell(grid.x_edges, xv)?;
    let r = cell(grid.y_edges, yv)?;
    if r >= grid.rows || c >= grid.cols {
        return None;
    }
    Some(Readout {
        name: series.name.clone(),
        x: xv,
        y: yv,
        z: grid.z.get(r * grid.cols + c).copied(),
        at: probe.at,
    })
}

/// Offer the points of a point series (line, stems, points) to the nearest
/// search. Other kinds offer nothing.
fn point_read(probe: &Probe, series: &PlotSeries, nearest: &mut Option<(f64, Readout)>) {
    let (xs, ys, line) = match &series.kind {
        PlotSeriesKind::Line { x, y } => (x.as_slice(), y.as_slice(), true),
        PlotSeriesKind::Stems { x, y } | PlotSeriesKind::Points { x, y } => {
            (x.as_slice(), y.as_slice(), false)
        }
        _ => return,
    };
    let Some(indices) = candidates(line, xs, ys, probe.pointer_x) else {
        return;
    };
    for i in indices {
        let (Some(sx), Some(sy)) = (probe.x.px(xs[i]), probe.y.px(ys[i])) else {
            continue;
        };
        let d = (sx - probe.at.x).hypot(sy - probe.at.y);
        if nearest.as_ref().is_none_or(|(current, _)| d < *current) {
            *nearest = Some((
                d,
                Readout {
                    name: series.name.clone(),
                    x: xs[i],
                    y: ys[i],
                    z: None,
                    at: Point::new(sx, sy),
                },
            ));
        }
    }
}

/// The indices a readout searches. A line gives the two points around
/// `pointer_x` (found by binary search) and none when its x values do not
/// ascend; the other kinds give every point.
fn candidates(line: bool, xs: &[f64], ys: &[f64], pointer_x: f64) -> Option<Vec<usize>> {
    let n = xs.len().min(ys.len());
    if !line {
        return Some((0..n).collect());
    }
    if !ascending(xs) {
        return None;
    }
    let next = xs.partition_point(|v| *v < pointer_x);
    Some(
        [next.checked_sub(1), Some(next)]
            .into_iter()
            .flatten()
            .filter(|i| *i < n)
            .collect(),
    )
}

/// Whether `values` never decrease and hold no NaN.
fn ascending(values: &[f64]) -> bool {
    values.iter().all(|v| !v.is_nan()) && values.windows(2).all(|pair| pair[0] <= pair[1])
}

#[cfg(test)]
mod tests {
    //! Role: Tests for zoom, pan, keys and readout arithmetic.

    use super::*;
    use crate::plot::axis::PlotAxis;

    fn area() -> Rect {
        Rect::new(0.0, 0.0, 100.0, 100.0)
    }

    fn view() -> PlotView {
        PlotView {
            x_min: 0.0,
            x_max: 10.0,
            y_min: 0.0,
            y_max: 10.0,
        }
    }

    /// A linear plot with one line through (0,0), (2,2), (8,8), (10,10).
    fn model() -> PlotModel {
        PlotModel::new(PlotAxis::new("x"), PlotAxis::new("y")).series(PlotSeries::line(
            "line",
            vec![0.0, 2.0, 8.0, 10.0],
            vec![0.0, 2.0, 8.0, 10.0],
        ))
    }

    fn manual(range: PlotRange) -> PlotView {
        match range {
            PlotRange::Manual(view) => view,
            PlotRange::Auto => panic!("expected a manual range"),
        }
    }

    #[test]
    fn zooming_keeps_the_anchor_in_place() {
        let (lo, hi) = zoom_range(0.0, 10.0, AxisScale::Linear, 0.25, 2.0);
        assert!((lo + 2.5).abs() < 1e-9 && (hi - 17.5).abs() < 1e-9);
        let before = 0.25 * 10.0;
        let after = lo + 0.25 * (hi - lo);
        assert!((before - after).abs() < 1e-9);
        let (lo, hi) = zoom_range(0.0, 10.0, AxisScale::Linear, 0.5, 0.5);
        assert!((lo - 2.5).abs() < 1e-9 && (hi - 7.5).abs() < 1e-9);
    }

    #[test]
    fn a_log_zoom_works_in_log_space() {
        let (lo, hi) = zoom_range(1.0, 100.0, AxisScale::Log10, 0.5, 2.0);
        assert!((lo - 0.1).abs() < 1e-9 && (hi - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn a_drag_moves_the_content_with_the_pointer() {
        // Dragging right shows smaller x; dragging down shows larger y.
        let moved = drag_view(&model(), view(), area(), Point::new(10.0, 10.0));
        assert!((moved.x_min + 1.0).abs() < 1e-9 && (moved.x_max - 9.0).abs() < 1e-9);
        assert!((moved.y_min - 1.0).abs() < 1e-9 && (moved.y_max - 11.0).abs() < 1e-9);
    }

    #[test]
    fn a_wheel_zooms_x_and_pans_by_horizontal_delta() {
        let zoomed = wheel_view(
            &model(),
            view(),
            area(),
            Point::new(50.0, 50.0),
            Point::new(0.0, 100.0),
            false,
        );
        assert!(zoomed.x_max - zoomed.x_min > 10.0);
        assert_eq!((zoomed.y_min, zoomed.y_max), (0.0, 10.0));
        let panned = wheel_view(
            &model(),
            view(),
            area(),
            Point::new(50.0, 50.0),
            Point::new(20.0, 0.0),
            false,
        );
        assert!(panned.x_min > 0.0 && (panned.x_max - panned.x_min - 10.0).abs() < 1e-9);
        let ctrl = wheel_view(
            &model(),
            view(),
            area(),
            Point::new(50.0, 50.0),
            Point::new(0.0, 100.0),
            true,
        );
        assert_eq!((ctrl.x_min, ctrl.x_max), (0.0, 10.0));
        assert!(ctrl.y_max - ctrl.y_min > 10.0);
    }

    #[test]
    fn a_key_zooms_pans_or_fits() {
        let narrower = manual(Nudge::ZoomX(1.0 / KEY_ZOOM).apply(&model(), view()));
        assert!((narrower.x_max - narrower.x_min - 10.0 / KEY_ZOOM).abs() < 1e-9);
        let right = manual(Nudge::PanX(KEY_PAN).apply(&model(), view()));
        assert!((right.x_min - 1.0).abs() < 1e-9 && (right.x_max - 11.0).abs() < 1e-9);
        let up = manual(Nudge::PanY(KEY_PAN).apply(&model(), view()));
        assert!((up.y_min - 1.0).abs() < 1e-9);
        assert_eq!(Nudge::Fit.apply(&model(), view()), PlotRange::Auto);
        assert_eq!(Nudge::of_key(Key::Char('x')), None);
        assert_eq!(Nudge::of_key(Key::Home), Some(Nudge::Fit));
    }

    #[test]
    fn the_readout_is_the_nearest_point() {
        // (2, 2) is at pixels (20, 80); the pointer is close to it.
        let read =
            readout(&model(), view(), area(), Point::new(22.0, 78.0)).expect("a point is near");
        assert_eq!((read.x, read.y), (2.0, 2.0));
        assert_eq!(read.name, "line");
        assert!(readout(&model(), view(), area(), Point::new(50.0, 50.0)).is_some());
    }

    #[test]
    fn a_descending_line_is_not_read() {
        let descending = PlotModel::new(PlotAxis::new("x"), PlotAxis::new("y")).series(
            PlotSeries::line("line", vec![10.0, 8.0, 2.0, 0.0], vec![10.0, 8.0, 2.0, 0.0]),
        );
        assert!(readout(&descending, view(), area(), Point::new(22.0, 78.0)).is_none());
    }
}
