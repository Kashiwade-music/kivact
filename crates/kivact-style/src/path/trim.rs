//! Role: Path length measurement and trimming a path by length.
//!
//! `Path::length` and `Path::trim`: a path measured, and a stretch of it
//! chosen by its length.
use super::Path;
use kurbo::{
    BezPath, CubicBez, Line, ParamCurve, ParamCurveArclen, PathEl, PathSeg, Point as KPoint,
    QuadBez,
};

/// The absolute error of a measured length, in px.
const ACCURACY: f64 = 1e-3;

/// One contour: where it starts, its segments (a closing edge included)
/// and whether it was closed.
struct Contour {
    start: KPoint,
    segs: Vec<PathSeg>,
    closed: bool,
    /// Whether the closing edge is the last of `segs` (it is not when the
    /// contour already ends where it starts).
    closing_edge: bool,
}

impl Contour {
    fn lengths(&self) -> Vec<f64> {
        self.segs.iter().map(|s| s.arclen(ACCURACY)).collect()
    }

    /// The whole contour, as it was written.
    fn emit_all(&self, out: &mut BezPath) {
        out.move_to(self.start);
        let drawn = self.segs.len() - usize::from(self.closing_edge);
        out.extend(self.segs[..drawn].iter().map(PathSeg::as_path_el));
        if self.closed {
            out.close_path();
        }
    }
}

fn contours(path: &BezPath) -> Vec<Contour> {
    let mut out = Vec::new();
    let mut open: Option<Contour> = None;
    let mut last = KPoint::ZERO;
    // Where the next contour starts: after a move, or after a close.
    let mut origin = KPoint::ZERO;
    for el in path.elements() {
        if let PathEl::MoveTo(p) = *el {
            out.extend(open.take().filter(|c| !c.segs.is_empty()));
            (last, origin) = (p, p);
            continue;
        }
        let contour = open.get_or_insert_with(|| Contour {
            start: origin,
            segs: Vec::new(),
            closed: false,
            closing_edge: false,
        });
        match *el {
            PathEl::LineTo(p) => {
                contour.segs.push(PathSeg::Line(Line::new(last, p)));
                last = p;
            }
            PathEl::QuadTo(a, b) => {
                contour.segs.push(PathSeg::Quad(QuadBez::new(last, a, b)));
                last = b;
            }
            PathEl::CurveTo(a, b, c) => {
                contour
                    .segs
                    .push(PathSeg::Cubic(CubicBez::new(last, a, b, c)));
                last = c;
            }
            PathEl::ClosePath => {
                if last != contour.start {
                    contour
                        .segs
                        .push(PathSeg::Line(Line::new(last, contour.start)));
                    contour.closing_edge = true;
                }
                contour.closed = true;
                (last, origin) = (contour.start, contour.start);
                out.extend(open.take().filter(|c| !c.segs.is_empty()));
            }
            PathEl::MoveTo(_) => {}
        }
    }
    out.extend(open.take().filter(|c| !c.segs.is_empty()));
    out
}

impl Path {
    /// The length of every contour added up, in px: a closed contour's
    /// closing edge counts, the distance between contours does not.
    ///
    /// ```
    /// use kivact::draw::{Path};
    /// use kivact::geometry::Point;
    /// let line = Path::new().move_to(Point::new(0.0, 0.0)).line_to(Point::new(30.0, 40.0));
    /// assert!((line.length() - 50.0).abs() < 1e-3);
    /// // A circle of radius 10 (the cubics are within 0.01 px of it).
    /// let circle = Path::new().arc(Point::new(0.0, 0.0), 10, 0, 360, false).close();
    /// assert!((circle.length() - 62.83).abs() < 0.05);
    /// ```
    pub fn length(&self) -> f64 {
        self.bez().map_or(0.0, |bez| {
            bez.segments()
                .map(|segment| segment.arclen(ACCURACY))
                .sum::<f64>()
        })
    }

    /// The part of the path between the fractions `start` and `end` of its
    /// length, turned by `offset` (SVG and After Effects' trim paths).
    ///
    /// All contours count as one line: the length of each, a closed
    /// contour's closing edge included, added up in order, the gaps between
    /// them not. `start` and `end` are clamped to `0..=1` (`end` before
    /// `start` swaps them), and `offset` moves both along the path, wrapping
    /// past the end around to the start. A closed contour that is the whole
    /// path continues across its first point, so a wrapped stretch is one
    /// line there; with several contours, or an open one, the stretch that
    /// wraps is two lines. The whole path (`0..1`) is returned as it is,
    /// closed contours included; any shorter stretch is open lines. Curves
    /// stay curves, and the fill rule is kept.
    ///
    /// Non-finite arguments, an empty stretch and a path without length give
    /// an empty path.
    ///
    /// ```
    /// use kivact::draw::{Path};
    /// use kivact::geometry::Point;
    /// let line = Path::new().move_to(Point::new(0.0, 0.0)).line_to(Point::new(100.0, 0.0));
    /// let part = line.trim(0.25, 0.5, 0.0);
    /// let b = part.bounds().unwrap();
    /// assert!((b.x - 25.0).abs() < 1e-3 && (b.width - 25.0).abs() < 1e-3);
    /// // Turned by three quarters, the stretch is the last quarter.
    /// let turned = line.trim(0.0, 0.25, 0.75);
    /// assert!((turned.length() - 25.0).abs() < 1e-3);
    /// ```
    pub fn trim(&self, start: f64, end: f64, offset: f64) -> Path {
        let empty = Path {
            cmds: Vec::new(),
            even_odd: self.even_odd,
        };
        if !(start.is_finite() && end.is_finite() && offset.is_finite()) {
            return empty;
        }
        let (start, end) = {
            let (a, b) = (start.clamp(0.0, 1.0), end.clamp(0.0, 1.0));
            (a.min(b), a.max(b))
        };
        let Some(bez) = self.bez() else { return empty };
        let contours = contours(&bez);
        let lengths: Vec<Vec<f64>> = contours.iter().map(Contour::lengths).collect();
        let total: f64 = lengths.iter().flatten().sum();
        if total <= 0.0 || !total.is_finite() || end <= start {
            return empty;
        }
        if start <= 0.0 && end >= 1.0 {
            let mut all = BezPath::new();
            contours.iter().for_each(|c| c.emit_all(&mut all));
            return Path::from_bez(&all, self.even_odd);
        }
        let begin = (start + offset).rem_euclid(1.0) * total;
        let stop = begin + (end - start) * total;
        let intervals: Vec<(f64, f64)> = if stop > total {
            vec![(begin, total), (0.0, stop - total)]
        } else {
            vec![(begin, stop)]
        };
        // A closed contour that is the whole path continues across its seam.
        let seam = contours.len() == 1 && contours[0].closed;
        let mut out = BezPath::new();
        for (n, (low, high)) in intervals.into_iter().enumerate() {
            let mut at = 0.0;
            for (contour, lengths) in contours.iter().zip(&lengths) {
                let length: f64 = lengths.iter().sum();
                let (from, to) = ((low - at).max(0.0), (high - at).min(length));
                if to > from {
                    cut(contour, lengths, from, to, &mut out, seam && n == 1);
                }
                at += length;
            }
        }
        Path::from_bez(&out, self.even_odd)
    }
}

/// The part of `contour` from `from` to `to` (lengths along it) as an open
/// line, continuing the line `out` ends with when `join`.
fn cut(contour: &Contour, lengths: &[f64], from: f64, to: f64, out: &mut BezPath, join: bool) {
    let mut at = 0.0;
    let mut first = !join;
    for (seg, length) in contour.segs.iter().zip(lengths) {
        let (a, b) = ((from - at).max(0.0), (to - at).min(*length));
        at += length;
        if b <= a {
            continue;
        }
        let ta = if a <= 0.0 {
            0.0
        } else {
            seg.inv_arclen(a, ACCURACY)
        };
        let tb = if b >= *length {
            1.0
        } else {
            seg.inv_arclen(b, ACCURACY)
        };
        let part = seg.subsegment(ta..tb);
        if first {
            out.move_to(part.start());
            first = false;
        }
        out.push(part.as_path_el());
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests for path trimming.
    use super::*;
    use crate::geometry::{Point, Rect};
    use kivact_types::PathCmd;

    fn moves(path: &Path) -> usize {
        path.cmds
            .iter()
            .filter(|c| matches!(c, PathCmd::MoveTo(_)))
            .count()
    }

    fn square() -> Path {
        Path::new().rect(Rect::new(0.0, 0.0, 10.0, 10.0))
    }

    #[test]
    fn a_wrapped_closed_path_is_one_line_across_its_seam() {
        let part = square().trim(0.0, 0.25, 0.875);
        assert_eq!(moves(&part), 1, "{:?}", part.cmds);
        assert!((part.length() - 10.0).abs() < 1e-3);
        assert_eq!(part.bounds(), Some(Rect::new(0.0, 0.0, 5.0, 5.0)));
    }

    #[test]
    fn an_open_path_that_wraps_is_two_lines() {
        let line = Path::new()
            .move_to(Point::new(0.0, 0.0))
            .line_to(Point::new(100.0, 0.0));
        assert_eq!(moves(&line.trim(0.0, 0.25, 0.9)), 2);
    }

    #[test]
    fn the_whole_path_is_kept_and_a_stretch_is_open() {
        assert_eq!(square().trim(0.0, 1.0, 0.3), square());
        let part = square().trim(0.0, 0.5, 0.0);
        assert!(!part.cmds.iter().any(|c| matches!(c, PathCmd::Close)));
        assert!((part.length() - 20.0).abs() < 1e-3);
    }

    #[test]
    fn an_empty_stretch_or_invalid_value_is_empty() {
        for (s, e, o) in [
            (0.5, 0.5, 0.0),
            (f64::NAN, 1.0, 0.0),
            (0.0, 0.5, f64::INFINITY),
        ] {
            assert!(square().trim(s, e, o).cmds.is_empty(), "{s} {e} {o}");
        }
        assert!(Path::new().trim(0.0, 0.5, 0.0).cmds.is_empty());
        assert_eq!(
            square().trim(0.8, 0.2, 0.0).length(),
            square().trim(0.2, 0.8, 0.0).length()
        );
    }

    #[test]
    fn contours_add_up_and_the_gap_between_them_does_not() {
        let two = square().append(&Path::new().rect(Rect::new(100.0, 0.0, 10.0, 10.0)));
        assert!((two.length() - 80.0).abs() < 1e-3);
        // The second contour starts at 0.5.
        let second = two.trim(0.5, 0.625, 0.0);
        assert_eq!(second.bounds(), Some(Rect::new(100.0, 0.0, 10.0, 0.0)));
    }
}
