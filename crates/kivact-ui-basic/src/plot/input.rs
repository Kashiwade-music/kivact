//! Role: Zoom, pan and readout arithmetic of a plot, free of events and signals.

use super::decimate::Bounds;
use super::heatmap::cell;
use super::model::{AxisScale, PlotModel, PlotSeriesKind, PlotView};
use super::scale::Map;

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
pub(super) fn pan_range(lo: f64, hi: f64, scale: AxisScale, shift: f64) -> (f64, f64) {
    let (a, b) = (transform(scale, lo), transform(scale, hi));
    let by = (b - a) * shift;
    (inverse(scale, a + by), inverse(scale, b + by))
}

fn transform(scale: AxisScale, value: f64) -> f64 {
    match scale {
        AxisScale::Linear => value,
        AxisScale::Log10 => value.log10(),
    }
}

fn inverse(scale: AxisScale, value: f64) -> f64 {
    match scale {
        AxisScale::Linear => value,
        AxisScale::Log10 => 10f64.powf(value),
    }
}

/// The pan of a view by a drag of `(dx, dy)` pixels in a plot of `bounds`:
/// the content follows the pointer, so a drag to the right shows smaller x,
/// and a drag down shows larger y.
pub(super) fn pan(model: &PlotModel, view: PlotView, bounds: Bounds, dx: f64, dy: f64) -> PlotView {
    let width = (bounds.right - bounds.left).max(1.0);
    let height = (bounds.bottom - bounds.top).max(1.0);
    let (x_min, x_max) = pan_range(view.x_min, view.x_max, model.x.scale, -dx / width);
    let (y_min, y_max) = pan_range(view.y_min, view.y_max, model.y.scale, dy / height);
    PlotView {
        x_min,
        x_max,
        y_min,
        y_max,
    }
}

/// The view zoomed about the pointer at `(px, py)` by `factor` on the y axis
/// (`vertical`) or the x axis.
pub(super) fn zoom(
    model: &PlotModel,
    view: PlotView,
    bounds: Bounds,
    (px, py): (f64, f64),
    vertical: bool,
    factor: f64,
) -> PlotView {
    if vertical {
        let anchor = fraction(py, bounds.bottom, bounds.top);
        let (y_min, y_max) = zoom_range(view.y_min, view.y_max, model.y.scale, anchor, factor);
        PlotView {
            y_min,
            y_max,
            ..view
        }
    } else {
        let anchor = fraction(px, bounds.left, bounds.right);
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
    pub(super) at: (f64, f64),
}

/// The maps of a view over `bounds`: x from left to right, y from bottom to top.
pub(super) fn maps(model: &PlotModel, view: PlotView, bounds: Bounds) -> (Map, Map) {
    let x = Map::new(
        model.x.scale,
        view.x_min,
        view.x_max,
        bounds.left,
        bounds.right - bounds.left,
    );
    let y = Map::new(
        model.y.scale,
        view.y_min,
        view.y_max,
        bounds.bottom,
        bounds.top - bounds.bottom,
    );
    (x, y)
}

/// The readout at the pointer `(px, py)`. Over a heatmap cell it reads the
/// cell. Otherwise it reads the point nearest to the pointer among the line,
/// point and stem series; a line is searched by binary search for its nearest
/// x, and only when its x values ascend. `None` when there is nothing to read.
pub(super) fn readout(
    model: &PlotModel,
    view: PlotView,
    bounds: Bounds,
    (px, py): (f64, f64),
) -> Option<Readout> {
    let (mx, my) = maps(model, view, bounds);
    let pointer_x = mx.value(px);
    let mut nearest: Option<(f64, Readout)> = None;
    let mut cell_read: Option<Readout> = None;
    for series in &model.series {
        match &series.kind {
            PlotSeriesKind::Line { x, y }
            | PlotSeriesKind::Stems { x, y }
            | PlotSeriesKind::Points { x, y, .. } => {
                let (xs, ys) = (x.as_slice(), y.as_slice());
                let indices: Vec<usize> = if matches!(series.kind, PlotSeriesKind::Line { .. }) {
                    if !ascending(xs) {
                        continue;
                    }
                    let next = xs.partition_point(|v| *v < pointer_x);
                    [next.checked_sub(1), Some(next)]
                        .into_iter()
                        .flatten()
                        .filter(|i| *i < xs.len().min(ys.len()))
                        .collect()
                } else {
                    (0..xs.len().min(ys.len())).collect()
                };
                for i in indices {
                    let (Some(sx), Some(sy)) = (mx.px(xs[i]), my.px(ys[i])) else {
                        continue;
                    };
                    let d = (sx - px).hypot(sy - py);
                    if nearest.as_ref().is_none_or(|(current, _)| d < *current) {
                        nearest = Some((
                            d,
                            Readout {
                                name: series.name.clone(),
                                x: xs[i],
                                y: ys[i],
                                z: None,
                                at: (sx, sy),
                            },
                        ));
                    }
                }
            }
            PlotSeriesKind::Heatmap {
                x_edges,
                y_edges,
                z,
                rows,
                cols,
                ..
            } => {
                let inside = px >= bounds.left
                    && px <= bounds.right
                    && py >= bounds.top
                    && py <= bounds.bottom;
                if !inside || cell_read.is_some() {
                    continue;
                }
                let (xv, yv) = (pointer_x, my.value(py));
                let (Some(c), Some(r)) =
                    (cell(x_edges.as_slice(), xv), cell(y_edges.as_slice(), yv))
                else {
                    continue;
                };
                let (rows, cols) = (*rows as usize, *cols as usize);
                if r >= rows || c >= cols {
                    continue;
                }
                cell_read = Some(Readout {
                    name: series.name.clone(),
                    x: xv,
                    y: yv,
                    z: z.as_slice().get(r * cols + c).copied(),
                    at: (px, py),
                });
            }
            PlotSeriesKind::Steps { .. } => {}
        }
    }
    cell_read.or_else(|| nearest.map(|(_, readout)| readout))
}

/// Whether `values` never decrease and hold no NaN.
fn ascending(values: &[f64]) -> bool {
    values.iter().all(|v| !v.is_nan()) && values.windows(2).all(|pair| pair[0] <= pair[1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plot::model::{
        LegendPosition, LineDash, PlotAxis, PlotColor, PlotData, PlotSeries, TickFormat,
    };

    fn bounds() -> Bounds {
        Bounds {
            left: 0.0,
            top: 0.0,
            right: 100.0,
            bottom: 100.0,
        }
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
        let axis = || PlotAxis {
            label: String::new(),
            scale: AxisScale::Linear,
            format: TickFormat::Plain,
        };
        PlotModel {
            title: String::new(),
            x: axis(),
            y: axis(),
            series: vec![PlotSeries {
                name: "line".into(),
                color: PlotColor::Palette(0),
                width: 1.0,
                dash: LineDash::Solid,
                kind: PlotSeriesKind::Line {
                    x: PlotData::new(vec![0.0, 2.0, 8.0, 10.0]),
                    y: PlotData::new(vec![0.0, 2.0, 8.0, 10.0]),
                },
            }],
            legend: LegendPosition::Hidden,
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
    fn a_pan_moves_the_content_with_the_pointer() {
        // Dragging right shows smaller x; dragging down shows larger y.
        let moved = pan(&model(), view(), bounds(), 10.0, 10.0);
        assert!((moved.x_min + 1.0).abs() < 1e-9 && (moved.x_max - 9.0).abs() < 1e-9);
        assert!((moved.y_min - 1.0).abs() < 1e-9 && (moved.y_max - 11.0).abs() < 1e-9);
    }

    #[test]
    fn the_readout_is_the_nearest_point() {
        // (2, 2) is at pixels (20, 80); the pointer is close to it.
        let read = readout(&model(), view(), bounds(), (22.0, 78.0)).expect("a point is near");
        assert_eq!((read.x, read.y), (2.0, 2.0));
        assert_eq!(read.name, "line");
        assert!(readout(&model(), view(), bounds(), (50.0, 50.0)).is_some());
    }

    #[test]
    fn a_descending_line_is_not_read() {
        let mut descending = model();
        if let PlotSeriesKind::Line { x, .. } = &mut descending.series[0].kind {
            *x = PlotData::new(vec![10.0, 8.0, 2.0, 0.0]);
        }
        assert!(readout(&descending, view(), bounds(), (22.0, 78.0)).is_none());
    }
}
