//! Role: Splits a line into runs and keeps one min and max per pixel column.

use super::scale::within;
use kivact::geometry::{Point, Rect};

/// A point in plot pixels; `None` breaks the line (a NaN, or a value the axis cannot show).
pub(super) type Pixel = Option<Point>;

/// The drawable runs of a line of `points` in the plot area `area`.
///
/// A break ends a run. Inside a run, the points outside `area` are kept
/// only when they are next to a point inside, so each run reaches just past
/// the visible area on either side. When the points inside are more than
/// twice the plot width in logical px, each pixel column keeps the points
/// of its lowest and highest y, in their original order.
pub(super) fn runs(points: &[Pixel], area: Rect) -> Vec<Vec<Point>> {
    let mut out = Vec::new();
    let mut run: Vec<Point> = Vec::new();
    for point in points {
        match point {
            Some(p) => run.push(*p),
            None => {
                if !run.is_empty() {
                    out.push(finish(&run, area));
                    run.clear();
                }
            }
        }
    }
    if !run.is_empty() {
        out.push(finish(&run, area));
    }
    out.retain(|run| !run.is_empty());
    out
}

/// One run with its outside points thinned, and its inside points decimated
/// when they are dense.
fn finish(run: &[Point], area: Rect) -> Vec<Point> {
    let inside: Vec<bool> = run.iter().map(|p| within(area, *p)).collect();
    let inside_count = inside.iter().filter(|flag| **flag).count();
    let keep: Vec<bool> = (0..run.len())
        .map(|i| inside[i] || (i > 0 && inside[i - 1]) || (i + 1 < run.len() && inside[i + 1]))
        .collect();
    let dense = inside_count as f64 > 2.0 * area.width;
    let mut out = Vec::new();
    let mut i = 0;
    while i < run.len() {
        if !keep[i] {
            i += 1;
        } else if !dense || !inside[i] {
            out.push(run[i]);
            i += 1;
        } else {
            let end = column_end(run, &keep, &inside, i, area);
            push_extremes(&mut out, &run[i..end]);
            i = end;
        }
    }
    out
}

/// The end (exclusive) of the column of consecutive inside points that
/// starts at `start`: the points whose x falls in the same pixel column.
fn column_end(run: &[Point], keep: &[bool], inside: &[bool], start: usize, area: Rect) -> usize {
    let column = (run[start].x - area.x).floor();
    let mut end = start;
    while end < run.len() && keep[end] && inside[end] && (run[end].x - area.x).floor() == column {
        end += 1;
    }
    end
}

/// Push the lowest and the highest point of a column, in their original order.
fn push_extremes(out: &mut Vec<Point>, column: &[Point]) {
    let mut low = 0;
    let mut high = 0;
    for (k, point) in column.iter().enumerate() {
        if point.y < column[low].y {
            low = k;
        }
        if point.y > column[high].y {
            high = k;
        }
    }
    let (first, second) = if low <= high {
        (low, high)
    } else {
        (high, low)
    };
    out.push(column[first]);
    if second != first {
        out.push(column[second]);
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests for line runs and min and max thinning.

    use super::*;

    fn area(width: f64) -> Rect {
        Rect::new(0.0, 0.0, width, 100.0)
    }

    fn at(x: f64, y: f64) -> Pixel {
        Some(Point::new(x, y))
    }

    #[test]
    fn a_break_splits_the_line() {
        let points = [at(1.0, 1.0), at(2.0, 2.0), None, at(3.0, 3.0)];
        let runs = runs(&points, area(10.0));
        assert_eq!(
            runs,
            vec![
                vec![Point::new(1.0, 1.0), Point::new(2.0, 2.0)],
                vec![Point::new(3.0, 3.0)],
            ]
        );
    }

    #[test]
    fn outside_points_keep_one_neighbour_on_each_side() {
        let points: Vec<Pixel> = [-50.0, -40.0, 5.0, 6.0, 200.0, 300.0]
            .iter()
            .map(|x| at(*x, 50.0))
            .collect();
        let runs = runs(&points, area(10.0));
        let xs: Vec<f64> = runs[0].iter().map(|p| p.x).collect();
        assert_eq!(xs, vec![-40.0, 5.0, 6.0, 200.0]);
    }

    #[test]
    fn a_dense_zigzag_keeps_two_points_per_column() {
        let width = 10.0;
        let points: Vec<Pixel> = (0..1000)
            .map(|i| {
                let x = f64::from(i) / 100.0;
                let y = if i % 2 == 0 { 10.0 } else { 90.0 };
                at(x, y)
            })
            .collect();
        let runs = runs(&points, area(width));
        assert_eq!(runs.len(), 1);
        assert!(runs[0].len() <= 2 * (width as usize + 1));
        let lows = runs[0].iter().filter(|p| p.y == 10.0).count();
        let highs = runs[0].iter().filter(|p| p.y == 90.0).count();
        assert!(lows > 0 && highs > 0);
    }

    #[test]
    fn a_sparse_line_is_kept_whole() {
        let points: Vec<Pixel> = (0..5).map(|i| at(f64::from(i), 50.0)).collect();
        let runs = runs(&points, area(100.0));
        assert_eq!(runs[0].len(), 5);
    }
}
