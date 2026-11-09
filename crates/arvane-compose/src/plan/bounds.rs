//! Role: Where each command of a render scene can draw, in scene coordinates.
//!
//! Ported from `arvane-render`'s backdrop index (`git show f4e40cc:crates/arvane-render/src/backdrop.rs`)
//! to the scene vocabulary of `kivact-types`: clips, transforms and groups
//! are the three kinds of nested group. The bounds are generous (strokes,
//! glyphs and blurs grow them) so the damage they give is never too small.

use kivact_types::Matrix4;
use kivact_types::geometry::{Point, Rect, Transform};
use kivact_types::render::{BoxShadow, Glyph, Group, RenderCmd, RenderScene, SHADOW_REACH};
use kivact_types::scene::{LineJoin, Shape, Stroke};

use super::ops::PlanError;

/// Blur reach in standard deviations (a group's blur grows it by 3σ).
const BLUR_REACH: f64 = 3.0;

/// The kind of group an open push starts.
enum Kind {
    /// A clip rectangle, in the coordinates of the push.
    Clip(Rect),
    /// A transform: the coordinates change inside.
    Transform,
    /// A group: its clip (bounds, in push coordinates) and blur.
    Group {
        // xtask-allow(state-fields): passive record - the group's clip bounds in push coordinates, absent when it has none
        clip: Option<Rect>,
        // xtask-allow(state-fields): passive record - the group's blur radius, absent when it has none
        blur: Option<f64>,
    },
}

/// An open group while indexing.
struct Open {
    /// Index of the push command.
    push: usize,
    /// The transform before the push.
    outer: Transform,
    /// The content's bounds so far (root coordinates of the scene).
    bounds: Option<Rect>,
    /// What the push opened.
    kind: Kind,
}

/// Per command, the bounds in the scene's coordinates (`None`: draws nothing).
/// A push's bounds cover its whole group; a pop has none.
pub(crate) fn scene_bounds(scene: &RenderScene) -> Result<Vec<Option<Rect>>, PlanError> {
    let mut out = vec![None; scene.commands.len()];
    let mut stack: Vec<Open> = Vec::new();
    let mut transform = Transform::IDENTITY;
    for (i, command) in scene.commands.iter().enumerate() {
        match command {
            RenderCmd::PushClip(r) => stack.push(open(i, transform, Kind::Clip(*r))),
            RenderCmd::PushGroup(g) => stack.push(open(i, transform, group_kind(g))),
            RenderCmd::PushTransform(t) => {
                stack.push(open(i, transform, Kind::Transform));
                transform = t.then(transform);
            }
            RenderCmd::PopClip | RenderCmd::PopGroup | RenderCmd::PopTransform => {
                let closed = stack
                    .pop()
                    .ok_or_else(|| PlanError::new("a pop without an open group"))?;
                if !matches_pop(&closed.kind, command) {
                    return Err(PlanError::new("groups cross: a pop closes the wrong group"));
                }
                transform = closed.outer;
                let bounds = close(&closed, transform);
                out[closed.push] = bounds;
                add(&mut stack, bounds);
            }
            _ => {
                let drawn = local_bounds(command).map(|r| transform.bounds(r));
                out[i] = drawn;
                add(&mut stack, drawn);
            }
        }
    }
    if !stack.is_empty() {
        return Err(PlanError::new("a group is left open at the end of a scene"));
    }
    Ok(out)
}

/// A new open group.
fn open(push: usize, outer: Transform, kind: Kind) -> Open {
    Open {
        push,
        outer,
        bounds: None,
        kind,
    }
}

/// The kind a group push opens.
fn group_kind(g: &Group) -> Kind {
    Kind::Group {
        clip: g.clip.as_ref().map(Shape::bounds),
        blur: g.blur,
    }
}

/// Whether a pop command closes a group of `kind`.
fn matches_pop(kind: &Kind, pop: &RenderCmd) -> bool {
    matches!(
        (kind, pop),
        (Kind::Clip(_), RenderCmd::PopClip)
            | (Kind::Transform, RenderCmd::PopTransform)
            | (Kind::Group { .. }, RenderCmd::PopGroup)
    )
}

/// The bounds of a closed group, in the coordinates of its push.
fn close(open: &Open, transform: Transform) -> Option<Rect> {
    match open.kind {
        Kind::Clip(r) => intersect_opt(open.bounds, transform.bounds(r)),
        Kind::Transform => open.bounds,
        Kind::Group { clip, blur } => {
            let blurred = match (open.bounds, blur) {
                (Some(b), Some(sigma)) => {
                    let reach = BLUR_REACH * sigma * transform.max_scale();
                    Some(b.inflate(reach, reach))
                }
                (bounds, _) => bounds,
            };
            match clip {
                Some(c) => intersect_opt(blurred, transform.bounds(c)),
                None => blurred,
            }
        }
    }
}

/// Add `bounds` to the innermost open group's.
fn add(stack: &mut [Open], bounds: Option<Rect>) {
    if let (Some(top), Some(b)) = (stack.last_mut(), bounds) {
        top.bounds = Some(top.bounds.map_or(b, |o| o.union(b)));
    }
}

/// The overlap of two optional bounds (`None` when either is missing or they do not meet).
fn intersect_opt(a: Option<Rect>, b: Rect) -> Option<Rect> {
    intersect(a?, b)
}

/// The overlap of `a` and `b` (`None`: they do not meet).
pub(crate) fn intersect(a: Rect, b: Rect) -> Option<Rect> {
    let (x0, y0) = (a.x.max(b.x), a.y.max(b.y));
    let (x1, y1) = (a.right().min(b.right()), a.bottom().min(b.bottom()));
    (x1 >= x0 && y1 >= y0).then_some(Rect {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    })
}

/// Where a drawing command can draw, in its own coordinates (generous for strokes, glyphs and blurs).
fn local_bounds(command: &RenderCmd) -> Option<Rect> {
    Some(match command {
        RenderCmd::Rect { rect, .. }
        | RenderCmd::RoundedRect { rect, .. }
        | RenderCmd::RoundedRectCorners { rect, .. }
        | RenderCmd::Image { rect, .. }
        | RenderCmd::ImageSampled { rect, .. } => *rect,
        RenderCmd::StrokeRect { rect, width, .. }
        | RenderCmd::StrokeRoundedRectCorners { rect, width, .. } => rect.inflate(*width, *width),
        RenderCmd::Line {
            from, to, width, ..
        } => points_bounds(&[*from, *to]).inflate(width + 1.0, width + 1.0),
        RenderCmd::Path { cmds, stroke, .. } => {
            let w = stroke.map_or(0.0, |(w, _)| w);
            Shape::Path {
                cmds: cmds.clone(),
                rule: Default::default(),
            }
            .bounds()
            .inflate(w, w)
        }
        RenderCmd::Shadow { rect, std_dev, .. } => {
            let r = SHADOW_REACH * std_dev;
            rect.inflate(r, r)
        }
        RenderCmd::Glyphs {
            run,
            origin,
            stroke,
            ..
        } => {
            let mut bounds = glyphs_bounds(&run.glyphs, run.font_size)?;
            bounds.x += origin.x;
            bounds.y += origin.y;
            match stroke {
                Some(outline) => bounds.inflate(outline.stroke.width, outline.stroke.width),
                None => bounds,
            }
        }
        RenderCmd::FillShape { shape, .. } => shape.bounds(),
        RenderCmd::StrokeShape { shape, stroke, .. } => {
            let w = stroke_reach(stroke);
            shape.bounds().inflate(w, w)
        }
        RenderCmd::BoxShadow(shadow) => box_shadow(shadow),
        RenderCmd::PushClip(_)
        | RenderCmd::PopClip
        | RenderCmd::PushTransform(_)
        | RenderCmd::PopTransform
        | RenderCmd::PushGroup(_)
        | RenderCmd::PopGroup => return None,
    })
}

/// How far a stroke reaches beyond its outline (joins included).
fn stroke_reach(stroke: &Stroke) -> f64 {
    let joints = if stroke.join == LineJoin::Miter {
        stroke.miter_limit.max(1.5)
    } else {
        1.5
    };
    stroke.width * joints
}

/// The bounds of a set of points.
pub(super) fn points_bounds(points: &[Point]) -> Rect {
    let mut x0 = f64::INFINITY;
    let mut y0 = f64::INFINITY;
    let mut x1 = f64::NEG_INFINITY;
    let mut y1 = f64::NEG_INFINITY;
    for p in points {
        x0 = x0.min(p.x);
        y0 = y0.min(p.y);
        x1 = x1.max(p.x);
        y1 = y1.max(p.y);
    }
    Rect {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    }
}

/// Glyphs' ink, generously: an em around each origin and its advance.
fn glyphs_bounds(glyphs: &[Glyph], size: f64) -> Option<Rect> {
    let first = glyphs.first()?;
    let mut b = (first.x, first.y, first.x, first.y);
    for g in glyphs {
        b = (
            b.0.min(g.x),
            b.1.min(g.y),
            b.2.max(g.x + g.advance.max(0.0)),
            b.3.max(g.y),
        );
    }
    Some(Rect {
        x: b.0 - size,
        y: b.1 - 1.5 * size,
        width: b.2 - b.0 + 2.0 * size,
        height: b.3 - b.1 + 2.5 * size,
    })
}

/// The bounds of a CSS box shadow (inset shadows stay in the box).
fn box_shadow(shadow: &BoxShadow) -> Rect {
    let rect = shadow.shape.rect;
    if shadow.inset {
        return rect;
    }
    let r = SHADOW_REACH * shadow.std_dev + shadow.spread.max(0.0);
    let cast = Rect {
        x: rect.x + shadow.offset.x,
        y: rect.y + shadow.offset.y,
        ..rect
    }
    .inflate(r, r);
    rect.union(cast)
}

/// Map a layer-space rectangle to window px bounds through a 4×4 map (None when nothing is in front of the viewer).
pub(crate) fn mapped_bounds(m: &Matrix4, r: Rect) -> Option<Rect> {
    m.flatten().map_rect(r)?.bounds()
}
