//! Role: Borders, shadows and the focus ring of a box.
//!
//! Invariants:
//! - A border lies inside the border box. One width and color on every
//!   side, solid, with circular corners is one stroke (as before borders
//!   had sides); other solid borders fill the ring between the border box
//!   and the padding box, each side's color in its trapezoid (the corners
//!   split on the diagonal, as CSS draws them). Dashed and dotted borders
//!   stroke the middle of each side's band with dashes three widths long
//!   or round dots one width wide.
//! - Box shadows follow the box's outline (elliptical corners too): outer
//!   ones beneath the background, cut out of the box when the background
//!   is not opaque (CSS); inset ones above the background. The first
//!   shadow is on top. Each is one `DrawCommand::BoxShadow` of CSS's
//!   `box-shadow`: the box grown by the spread (Arvane grows the radii by
//!   CSS's formula, so sharp corners stay sharp), moved by the offset,
//!   blurred by a gaussian of standard deviation blur / 2 (a blur of 0 is a
//!   hard edge); an inset one lies inside the box. A background is opaque
//!   only as one opaque color with no fade and no color filter; a shadow
//!   whose color is transparent is left out.
//! - The focus ring is a band of its width outside the border box,
//!   following its corners (or, with a clip path, the clip path's
//!   outline).

use crate::effects::Tint;
use crate::shape::BoxShape;
use kivact_types::DrawCommand as D;
use kivact_wire::{Border, BorderStyle, Edges, Fill, Ring, Shadow};
use kivact_wire::{Rect, Rgba};

use kivact_types::{FillRule, LineCap, Point, Shape, Stroke};

/// A border's sides in drawing order: top, right, bottom, left.
fn sides<T: Copy>(e: &Edges<T>) -> [T; 4] {
    [e.top, e.right, e.bottom, e.left]
}

/// The trapezoid of side `i` (top, right, bottom, left) between the
/// border box `o` and the padding box `p`.
fn trapezoid(i: usize, o: Rect, p: Rect) -> Shape {
    let pt = |x, y| Point { x, y };
    let (ol, ot, or, ob) = (o.x, o.y, o.right(), o.bottom());
    let (pl, ptop, pr, pb) = (p.x, p.y, p.right(), p.bottom());
    let points = match i {
        0 => vec![pt(ol, ot), pt(or, ot), pt(pr, ptop), pt(pl, ptop)],
        1 => vec![pt(or, ot), pt(or, ob), pt(pr, pb), pt(pr, ptop)],
        2 => vec![pt(or, ob), pt(ol, ob), pt(pl, pb), pt(pr, pb)],
        _ => vec![pt(ol, ob), pt(ol, ot), pt(pl, ptop), pt(pl, pb)],
    };
    Shape::Polygon {
        points,
        rule: FillRule::NonZero,
    }
}

/// The ring between `outer` and `inner` as one even-odd shape.
fn ring(outer: &BoxShape, inner: &BoxShape) -> Shape {
    let mut cmds = outer.shape().outline();
    cmds.extend(inner.shape().outline());
    Shape::Path {
        cmds,
        rule: FillRule::EvenOdd,
    }
}

/// Renderer strokes straddle the path; borders and rings lie inside
/// `rect`.
fn push_stroke(
    out: &mut kivact_types::Scene,
    rect: Rect,
    radii: [f32; 4],
    width: f32,
    color: Rgba,
) {
    let half = width / 2.0;
    let rect = Rect {
        x: rect.x + half,
        y: rect.y + half,
        width: (rect.width - width).max(0.0),
        height: (rect.height - width).max(0.0),
    };
    let radii = radii.map(|radius| (radius - half).max(0.0));
    if radii.iter().all(|radius| *radius == radii[0]) {
        out.commands.push(D::StrokeRect {
            rect,
            radius: radii[0],
            width,
            color,
        });
    } else {
        out.commands.push(D::StrokeRoundedRectCorners {
            rect,
            radii,
            width,
            color,
        });
    }
}

/// A band of `width` inside `shape` stroked in `color`, dashed as `style`
/// says (the band's middle line, `shape` inset by half the width).
fn stroke_band(
    out: &mut kivact_types::Scene,
    shape: &BoxShape,
    width: f32,
    style: BorderStyle,
    color: Rgba,
) {
    if style == BorderStyle::Solid
        && let BoxShape::Rect(r) | BoxShape::Circular(r, _) = shape
    {
        let radii = match shape {
            BoxShape::Circular(_, radii) => *radii,
            _ => [0.0; 4],
        };
        push_stroke(out, *r, radii, width, color);
        return;
    }
    let middle = shape.inset(Edges::all(width / 2.0));
    let stroke = match style {
        BorderStyle::Solid => Stroke::new(width),
        BorderStyle::Dashed => Stroke {
            dash: vec![3.0 * width, 3.0 * width],
            ..Stroke::new(width)
        },
        BorderStyle::Dotted => Stroke {
            cap: LineCap::Round,
            dash: vec![0.0, 2.0 * width],
            ..Stroke::new(width)
        },
    };
    out.commands.push(D::StrokeShape {
        shape: middle.shape(),
        stroke,
        paint: kivact_types::Paint::Solid(color),
    });
}

/// Draw `border` inside `shape` (see the module invariants).
pub fn border(out: &mut kivact_types::Scene, shape: &BoxShape, border: &Border, tint: &Tint) {
    let widths = crate::shape::border_widths(Some(border));
    let w = sides(&widths);
    if w.iter().all(|w| *w <= 0.0) {
        return;
    }
    let colors = sides(&border.colors);
    let shown: Vec<usize> = (0..4).filter(|i| w[*i] > 0.0).collect();
    let one_color = shown.iter().all(|i| colors[*i] == colors[shown[0]]);
    let color = |i: usize| tint.color(colors[i]);
    if one_color && w.iter().all(|x| *x == w[0]) {
        // The same band all around.
        let solid_corners = !matches!(shape, BoxShape::Rounded(_));
        if border.style != BorderStyle::Solid || solid_corners {
            stroke_band(out, shape, w[0], border.style, color(0));
            return;
        }
    }
    let inner = shape.inset(widths);
    let (o, p) = (shape.rect(), inner.rect());
    if border.style == BorderStyle::Solid {
        let band = ring(shape, &inner);
        if one_color {
            out.commands.push(D::FillShape {
                shape: band,
                paint: kivact_types::Paint::Solid(color(shown[0])),
            });
            return;
        }
        for i in shown {
            out.commands.push(D::PushLayer(kivact_types::Layer {
                clip: Some(trapezoid(i, o, p)),
                ..kivact_types::Layer::default()
            }));
            out.commands.push(D::FillShape {
                shape: band.clone(),
                paint: kivact_types::Paint::Solid(color(i)),
            });
            out.commands.push(D::PopLayer);
        }
        return;
    }
    // Dashed or dotted sides of their own widths.
    for i in shown {
        out.commands.push(D::PushLayer(kivact_types::Layer {
            clip: Some(trapezoid(i, o, p)),
            ..kivact_types::Layer::default()
        }));
        stroke_band(out, shape, w[i], border.style, color(i));
        out.commands.push(D::PopLayer);
    }
}

/// Whether `fill` hides everything beneath the box (one opaque color).
pub fn opaque(fill: Option<&Fill>, tint: &Tint) -> bool {
    matches!(fill, Some(Fill::Solid(c)) if c.a >= 1.0) && tint.alpha >= 1.0 && tint.matrix.is_none()
}

/// The box shadows of `shadows` that are (`inset`) or are not inset, the
/// last first (the first shadow ends on top). `knockout`: leave the box
/// out of outer shadows.
pub fn shadows(
    out: &mut kivact_types::Scene,
    shape: &BoxShape,
    shadows: &[Shadow],
    inset: bool,
    knockout: bool,
    tint: &Tint,
) {
    let rounded = shape.rounded();
    for s in shadows.iter().rev().filter(|s| s.inset == inset) {
        let color = tint.color(s.color);
        if color.a <= 0.0 {
            continue;
        }
        out.commands.push(D::BoxShadow(kivact_types::BoxShadow {
            shape: rounded,
            offset: s.offset,
            spread: s.spread,
            std_dev: s.blur / 2.0,
            color,
            inset,
            knockout: knockout && !inset,
        }));
    }
}

/// The focus ring around `shape` (or around `clip`, a clip path's
/// outline, which the ring straddles beneath the node's paint).
pub fn focus_ring(
    out: &mut kivact_types::Scene,
    shape: &BoxShape,
    clip: Option<&Shape>,
    ring_: &Ring,
    tint: &Tint,
) {
    let width = ring_.width;
    if width <= 0.0 {
        return;
    }
    let color = tint.color(ring_.color);
    if let Some(clip) = clip {
        out.commands.push(D::StrokeShape {
            shape: clip.clone(),
            stroke: Stroke::new(2.0 * width),
            paint: kivact_types::Paint::Solid(color),
        });
        return;
    }
    let outer = shape.grown(width);
    match outer {
        BoxShape::Rect(r) => push_stroke(out, r, [0.0; 4], width, color),
        BoxShape::Circular(r, radii) => push_stroke(out, r, radii, width, color),
        BoxShape::Rounded(_) => out.commands.push(D::FillShape {
            shape: ring(&outer, shape),
            paint: kivact_types::Paint::Solid(color),
        }),
    }
}
