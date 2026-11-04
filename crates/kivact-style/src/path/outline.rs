//! Role: `Path::stroke_outline`: a line's outline as a path to fill.
use super::Path;
use kivact_types::{LineCap, LineJoin, Stroke};
use kurbo::{Cap, Join, StrokeOpts};

/// The largest distance, in px, between the outline and the true stroke.
const TOLERANCE: f64 = 0.02;
/// The length of a dot's segment: a zero-length dash with a round or square
/// cap is a dot, and the stroker draws nothing for a zero-length segment.
const DOT: f64 = 1e-3;

impl Path {
    /// The outline of this path drawn with `stroke`, as a path to fill:
    /// `fill_path(&p.stroke_outline(&s), paint)` covers what
    /// `stroke_path(&p, &s, paint)` covers (width, caps, joins, miter limit
    /// and dashes), and the outline can be transformed, trimmed, measured or
    /// used as a clip like any other path. It is filled with the default
    /// `FillRule::NonZero`.
    ///
    /// A stroke that draws nothing (a width that is not positive and finite,
    /// a miter limit below 1, a dash that is not finite, negative or all
    /// zero) and a path without length give an empty path. A contour of
    /// zero length has no outline.
    ///
    /// ```
    /// use kivact::draw::{Path, Point};
    /// use kivact::draw::Stroke;
    /// let line = Path::new().move_to(Point::new(0.0, 5.0)).line_to(Point::new(20.0, 5.0));
    /// let band = line.stroke_outline(&Stroke::new(4));
    /// let b = band.bounds().unwrap();
    /// assert!((b.y - 3.0).abs() < 1e-3 && (b.height - 4.0).abs() < 1e-3);
    /// ```
    pub fn stroke_outline(&self, stroke: &Stroke) -> Path {
        let width = stroke.width;
        let valid = width.is_finite()
            && width > 0.0
            && stroke.miter_limit.is_finite()
            && stroke.miter_limit >= 1.0
            && stroke.dash_offset.is_finite()
            && stroke.dash.iter().all(|d| d.is_finite() && *d >= 0.0)
            && (stroke.dash.is_empty() || stroke.dash.iter().sum::<f64>() > 0.0);
        let Some(bez) = self.bez().filter(|_| valid) else {
            return Path::new();
        };
        let cap = match stroke.cap {
            LineCap::Butt => Cap::Butt,
            LineCap::Round => Cap::Round,
            LineCap::Square => Cap::Square,
        };
        let join = match stroke.join {
            LineJoin::Miter => Join::Miter,
            LineJoin::Round => Join::Round,
            LineJoin::Bevel => Join::Bevel,
        };
        let mut style = kurbo::Stroke::new(width)
            .with_caps(cap)
            .with_join(join)
            .with_miter_limit(stroke.miter_limit);
        if !stroke.dash.is_empty() {
            let mut dash: Vec<f64> = stroke.dash.to_vec();
            if dash.len() % 2 == 1 {
                dash.extend_from_within(..);
            }
            if stroke.cap != LineCap::Butt {
                for i in (0..dash.len()).step_by(2) {
                    if dash[i] == 0.0 {
                        dash[i] = DOT;
                        dash[i + 1] = (dash[i + 1] - DOT).max(0.0);
                    }
                }
            }
            style = style.with_dashes(stroke.dash_offset, dash);
        }
        let outline = kurbo::stroke(
            bez.elements().iter().copied(),
            &style,
            &StrokeOpts::default(),
            TOLERANCE,
        );
        Path::from_bez(&outline, false)
    }
}
