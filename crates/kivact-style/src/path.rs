//! Role: Reusable canvas paths in the shared shape vocabulary.
//!
//! [`Path`] is the one reusable path builder: `move_to`, `line_to`,
//! `quad_to`, `cubic_to`, centered `arc`, SVG `arc_to`, `ellipse`, `rect`,
//! `rounded_rect` with elliptical radii per corner, `close` and a
//! [`FillRule`]. It is drawn with `fill_path` and `stroke_path`. Its
//! operations use kurbo 0.13.1 (`ParamCurveArclen`, subsegments, strokes)
//! without making any kurbo type public; there is no flattening into
//! polygons for drawing and no boolean operation.
//!
//! - `append` is `Path2D.addPath`; `transformed` maps control points and
//!   keeps curves (arcs and ellipses become cubics first; a percentage
//!   translation is of a zero-sized box, so it is 0); `bounds` are the
//!   curves' bounds, not the control points', and `None` when empty.
//! - `length` counts closing edges but not the gaps between contours.
//!   `trim` cuts all contours as one line by `0..1` and turns it by an
//!   offset; only a single closed contour runs on past its start, the full
//!   range returns the path as written, and anything else returns open
//!   lines.
//! - `stroke_outline` is the outline the same cap, join, miter and dash
//!   would draw, as a path to fill with `NonZero`.
//! - Arcs become cubics as in `Shape::outline`: with a current point an
//!   arc draws a line to its start, without one it starts a contour.
mod commands;
mod geometry;
mod outline;
mod trim;

/// The rule used to fill a path's nested or intersecting contours.
#[doc(inline)]
pub use kivact_types::FillRule;

pub use commands::Path;
pub use geometry::PathContour;
