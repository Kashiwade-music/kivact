//! Role: Splits a line into runs and keeps one min and max per pixel column.

/// A point in plot pixels; `None` breaks the line (a NaN, or a value the axis cannot show).
pub(super) type Pixel = Option<(f64, f64)>;

/// The plot rectangle in pixels: left, top, right and bottom.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Bounds {
    /// The left edge.
    pub(super) left: f64,
    /// The top edge.
    pub(super) top: f64,
    /// The right edge.
    pub(super) right: f64,
    /// The bottom edge.
    pub(super) bottom: f64,
}

impl Bounds {
    /// Whether `(x, y)` lies inside, edges included.
    fn contains(&self, (x, y): (f64, f64)) -> bool {
        x >= self.left && x <= self.right && y >= self.top && y <= self.bottom
    }
}

/// The drawable runs of a line of `points`.
///
/// A break ends a run. Inside a run, the points outside `bounds` are kept
/// only when they are next to a point inside, so each run reaches just past
/// the visible area on either side. When the points inside are more than
/// twice the plot width in logical px, each pixel column keeps the points
/// of its lowest and highest y, in their original order.
pub(super) fn runs(points: &[Pixel], bounds: Bounds) -> Vec<Vec<(f64, f64)>> {
    let mut out = Vec::new();
    let mut run: Vec<(f64, f64)> = Vec::new();
    for point in points {
        match point {
            Some(p) => run.push(*p),
            None => {
                if !run.is_empty() {
                    out.push(finish(&run, bounds));
                    run.clear();
                }
            }
        }
    }
    if !run.is_empty() {
        out.push(finish(&run, bounds));
    }
    out.retain(|run| !run.is_empty());
    out
}

/// One run with its outside points thinned, and its inside points decimated
/// when they are dense.
fn finish(run: &[(f64, f64)], bounds: Bounds) -> Vec<(f64, f64)> {
    let inside: Vec<bool> = run.iter().map(|p| bounds.contains(*p)).collect();
    let inside_count = inside.iter().filter(|flag| **flag).count();
    let keep: Vec<bool> = (0..run.len())
        .map(|i| inside[i] || (i > 0 && inside[i - 1]) || (i + 1 < run.len() && inside[i + 1]))
        .collect();
    let width = bounds.right - bounds.left;
    let dense = inside_count as f64 > 2.0 * width;
    let mut out = Vec::new();
    let mut i = 0;
    while i < run.len() {
        if !keep[i] {
            i += 1;
        } else if !dense || !inside[i] {
            out.push(run[i]);
            i += 1;
        } else {
            // A column of consecutive inside points: keep its extremes.
            let column = (run[i].0 - bounds.left).floor();
            let mut end = i;
            while end < run.len()
                && keep[end]
                && inside[end]
                && (run[end].0 - bounds.left).floor() == column
            {
                end += 1;
            }
            let mut low = i;
            let mut high = i;
            for k in i..end {
                if run[k].1 < run[low].1 {
                    low = k;
                }
                if run[k].1 > run[high].1 {
                    high = k;
                }
            }
            let (first, second) = if low <= high {
                (low, high)
            } else {
                (high, low)
            };
            out.push(run[first]);
            if second != first {
                out.push(run[second]);
            }
            i = end;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    //! Role: Tests for line runs and min and max thinning.

    use super::*;

    fn bounds(width: f64) -> Bounds {
        Bounds {
            left: 0.0,
            top: 0.0,
            right: width,
            bottom: 100.0,
        }
    }

    #[test]
    fn a_break_splits_the_line() {
        let points = [Some((1.0, 1.0)), Some((2.0, 2.0)), None, Some((3.0, 3.0))];
        let runs = runs(&points, bounds(10.0));
        assert_eq!(runs, vec![vec![(1.0, 1.0), (2.0, 2.0)], vec![(3.0, 3.0)]]);
    }

    #[test]
    fn outside_points_keep_one_neighbour_on_each_side() {
        let points: Vec<Pixel> = [-50.0, -40.0, 5.0, 6.0, 200.0, 300.0]
            .iter()
            .map(|x| Some((*x, 50.0)))
            .collect();
        let runs = runs(&points, bounds(10.0));
        let xs: Vec<f64> = runs[0].iter().map(|p| p.0).collect();
        assert_eq!(xs, vec![-40.0, 5.0, 6.0, 200.0]);
    }

    #[test]
    fn a_dense_zigzag_keeps_two_points_per_column() {
        let width = 10.0;
        let points: Vec<Pixel> = (0..1000)
            .map(|i| {
                let x = f64::from(i) / 100.0;
                let y = if i % 2 == 0 { 10.0 } else { 90.0 };
                Some((x, y))
            })
            .collect();
        let runs = runs(&points, bounds(width));
        assert_eq!(runs.len(), 1);
        assert!(runs[0].len() <= 2 * (width as usize + 1));
        let lows = runs[0].iter().filter(|p| p.1 == 10.0).count();
        let highs = runs[0].iter().filter(|p| p.1 == 90.0).count();
        assert!(lows > 0 && highs > 0);
    }

    #[test]
    fn a_sparse_line_is_kept_whole() {
        let points: Vec<Pixel> = (0..5).map(|i| Some((f64::from(i), 50.0))).collect();
        let runs = runs(&points, bounds(100.0));
        assert_eq!(runs[0].len(), 5);
    }
}
