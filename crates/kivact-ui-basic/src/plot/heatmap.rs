//! Role: Heatmap painting: the cell of each pixel, and its color on the colormap.

use super::scale::Map;
use kivact::draw::{Draw, ImageOptions, Pixels};
use kivact::geometry::Rect;

/// The cell of `value` among `edges` (ascending or descending): the index
/// `i` with `edges[i]` and `edges[i + 1]` on either side of it. A value on
/// the last edge is in the last cell. `None` outside the edges or for NaN.
pub(super) fn cell(edges: &[f64], value: f64) -> Option<usize> {
    let n = edges.len();
    if n < 2 || value.is_nan() {
        return None;
    }
    let ascending = edges[0] <= edges[n - 1];
    let (low, high) = if ascending {
        (edges[0], edges[n - 1])
    } else {
        (edges[n - 1], edges[0])
    };
    if value < low || value > high {
        return None;
    }
    let passed = if ascending {
        edges.partition_point(|edge| *edge <= value)
    } else {
        edges.partition_point(|edge| *edge >= value)
    };
    Some(passed.saturating_sub(1).min(n - 2))
}

/// The color of `z` on `stops` (RGBA in `0..=1`, evenly spaced from `z_min`
/// to `z_max`). NaN is transparent.
fn color(z: f64, z_min: f64, z_max: f64, stops: &[[f64; 4]]) -> [u8; 4] {
    if !z.is_finite() || stops.is_empty() {
        return [0, 0, 0, 0];
    }
    let span = z_max - z_min;
    let t = if span > 0.0 {
        ((z - z_min) / span).clamp(0.0, 1.0)
    } else {
        0.5
    };
    let last = stops.len() - 1;
    let position = t * last as f64;
    let index = (position.floor() as usize).min(last.saturating_sub(1));
    let (a, b) = (stops[index], stops[(index + 1).min(last)]);
    let frac = if last == 0 {
        0.0
    } else {
        (position - index as f64).clamp(0.0, 1.0)
    };
    let mut out = [0u8; 4];
    for channel in 0..4 {
        let v = a[channel] + (b[channel] - a[channel]) * frac;
        out[channel] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
    out
}

/// The cell values of a heatmap, with its edges and value range.
pub(super) struct Cells<'a> {
    /// The column edges (`cols + 1`).
    pub(super) x_edges: &'a [f64],
    /// The row edges (`rows + 1`).
    pub(super) y_edges: &'a [f64],
    /// The values, row by row.
    pub(super) z: &'a [f64],
    /// The number of rows.
    pub(super) rows: usize,
    /// The number of columns.
    pub(super) cols: usize,
    /// The value of the first color.
    pub(super) z_min: f64,
    /// The value of the last color.
    pub(super) z_max: f64,
}

/// Paint `cells` into `rect` on the colormap `stops`: one image with a pixel
/// per logical px, each pixel taking the color of the cell under its center.
pub(super) fn paint(
    d: &mut Draw,
    rect: Rect,
    x: &Map,
    y: &Map,
    cells: &Cells<'_>,
    stops: &[[f64; 4]],
) {
    let width = rect.width.ceil().max(1.0) as u32;
    let height = rect.height.ceil().max(1.0) as u32;
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    for row in 0..height {
        let row_cell = cell(cells.y_edges, y.value(rect.y + f64::from(row) + 0.5));
        for col in 0..width {
            let col_cell = cell(cells.x_edges, x.value(rect.x + f64::from(col) + 0.5));
            let pixel = match (row_cell, col_cell) {
                (Some(r), Some(c)) if r < cells.rows && c < cells.cols => {
                    match cells.z.get(r * cells.cols + c) {
                        Some(z) => color(*z, cells.z_min, cells.z_max, stops),
                        None => [0, 0, 0, 0],
                    }
                }
                _ => [0, 0, 0, 0],
            };
            rgba.extend_from_slice(&pixel);
        }
    }
    let options = ImageOptions {
        pixelated: true,
        ..ImageOptions::default()
    };
    d.pixels(&Pixels::new(width, height, rgba), rect, options);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_value_finds_its_cell_in_either_edge_order() {
        let up = [0.0, 1.0, 2.0, 4.0];
        assert_eq!(cell(&up, 0.5), Some(0));
        assert_eq!(cell(&up, 1.0), Some(1));
        assert_eq!(cell(&up, 4.0), Some(2));
        assert_eq!(cell(&up, 4.5), None);
        assert_eq!(cell(&up, f64::NAN), None);
        let down = [4.0, 2.0, 1.0, 0.0];
        assert_eq!(cell(&down, 1.5), Some(1));
        assert_eq!(cell(&down, 0.0), Some(2));
    }

    #[test]
    fn colors_follow_the_stops_and_nan_is_transparent() {
        let stops = [[0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]];
        assert_eq!(color(0.0, 0.0, 1.0, &stops), [0, 0, 0, 255]);
        assert_eq!(color(1.0, 0.0, 1.0, &stops), [255, 255, 255, 255]);
        assert_eq!(color(f64::NAN, 0.0, 1.0, &stops), [0, 0, 0, 0]);
        assert_eq!(color(0.5, 0.0, 1.0, &stops), [128, 128, 128, 255]);
    }
}
