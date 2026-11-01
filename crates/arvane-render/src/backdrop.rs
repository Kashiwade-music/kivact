//! Role: Replay plan that tells a backdrop which earlier commands reach its area.
//!
//! Where a scene's commands draw, for backdrops (`DrawCommand::Backdrop`):
//! a backdrop draws again the commands before it that reach its area, and
//! skips whole groups that do not, so its cost follows what it replays.
//!
//! Built once per frame, and only for scenes with a backdrop: each
//! command's bounds in root coordinates, each group's content bounds and
//! its matching pop ([`Index`]). A replay costs the commands that reach
//! the backdrop's area plus the siblings it walks past.

use kivact_types::protocol::{
    BoxShadow, DrawCommand, Glyph, LayerFilter, SHADOW_REACH, Scene, Shape, Transform,
};
use kivact_types::{LineJoin, Rect};

/// Each command's bounds in the scene's root coordinates (logical pixels)
/// and each group's extent.
pub(crate) struct Index {
    /// A drawing command's bounds, a push's whole group's (`None`: draws
    /// nothing).
    pub(crate) bounds: Vec<Option<Rect>>,
    /// A push's matching pop (other commands: their own index).
    pub(crate) close: Vec<usize>,
}

/// An open group while indexing.
struct Open {
    push: usize,
    /// The transform before the push.
    outer: Transform,
    /// The content's bounds so far.
    bounds: Option<Rect>,
}

impl Index {
    /// The index of a valid scene (balanced groups).
    pub(crate) fn new(scene: &Scene) -> Self {
        let n = scene.commands.len();
        let mut bounds = vec![None; n];
        let mut close: Vec<usize> = (0..n).collect();
        let mut stack: Vec<Open> = Vec::new();
        let mut transform = Transform::IDENTITY;
        for (i, command) in scene.commands.iter().enumerate() {
            match command {
                DrawCommand::PushClip(_) | DrawCommand::PushLayer(_) => stack.push(Open {
                    push: i,
                    outer: transform,
                    bounds: None,
                }),
                DrawCommand::PushTransform(t) => {
                    stack.push(Open {
                        push: i,
                        outer: transform,
                        bounds: None,
                    });
                    transform = t.then(transform);
                }
                DrawCommand::PopClip | DrawCommand::PopTransform | DrawCommand::PopLayer => {
                    let open = stack.pop().expect("invariant: validated groups");
                    close[open.push] = i;
                    transform = open.outer;
                    let group = match &scene.commands[open.push] {
                        DrawCommand::PushClip(r) => {
                            open.bounds.and_then(|b| intersect(b, transform.bounds(*r)))
                        }
                        DrawCommand::PushLayer(layer) => {
                            let reach =
                                |std_dev: f32| SHADOW_REACH * std_dev * transform.max_scale();
                            let filtered = open.bounds.map(|b| match layer.filter {
                                None => b,
                                Some(LayerFilter::Blur { std_dev }) => {
                                    let r = reach(std_dev);
                                    b.inflate(r, r)
                                }
                                Some(LayerFilter::DropShadow {
                                    offset, std_dev, ..
                                }) => {
                                    let r = reach(std_dev);
                                    // The offset in root coordinates.
                                    let (o, z) = (
                                        transform.apply(offset),
                                        transform.apply(kivact_types::Point::default()),
                                    );
                                    b.union(
                                        Rect::new(
                                            b.x + o.x - z.x,
                                            b.y + o.y - z.y,
                                            b.width,
                                            b.height,
                                        )
                                        .inflate(r, r),
                                    )
                                }
                            });
                            match &layer.clip {
                                Some(clip) => filtered
                                    .and_then(|b| intersect(b, transform.bounds(clip.bounds()))),
                                None => filtered,
                            }
                        }
                        _ => open.bounds,
                    };
                    bounds[open.push] = group;
                    add(&mut stack, group);
                }
                _ => {
                    let local = local_bounds(command);
                    let drawn = local.map(|r| transform.bounds(r));
                    bounds[i] = drawn;
                    add(&mut stack, drawn);
                }
            }
        }
        Index { bounds, close }
    }
}

/// Add `bounds` to the innermost open group's.
fn add(stack: &mut [Open], bounds: Option<Rect>) {
    if let (Some(open), Some(b)) = (stack.last_mut(), bounds) {
        open.bounds = Some(open.bounds.map_or(b, |o| o.union(b)));
    }
}

/// The overlap of `a` and `b` (`None`: they do not meet).
pub(crate) fn intersect(a: Rect, b: Rect) -> Option<Rect> {
    let (x0, y0) = (a.x.max(b.x), a.y.max(b.y));
    let (x1, y1) = (a.right().min(b.right()), a.bottom().min(b.bottom()));
    (x1 >= x0 && y1 >= y0).then_some(Rect::new(x0, y0, x1 - x0, y1 - y0))
}

/// Where a drawing command can draw, in its own coordinates (generous
/// for strokes, glyphs and blurs).
fn local_bounds(command: &DrawCommand) -> Option<Rect> {
    Some(match command {
        DrawCommand::Rect { rect, .. }
        | DrawCommand::RoundedRect { rect, .. }
        | DrawCommand::RoundedRectCorners { rect, .. }
        | DrawCommand::Image { rect, .. }
        | DrawCommand::ImageSampled { rect, .. }
        | DrawCommand::External { rect, .. } => *rect,
        DrawCommand::StrokeRect { rect, width, .. }
        | DrawCommand::StrokeRoundedRectCorners { rect, width, .. } => rect.inflate(*width, *width),
        DrawCommand::Line {
            from, to, width, ..
        } => points(&[*from, *to]).inflate(width + 1.0, width + 1.0),
        DrawCommand::Path { cmds, stroke, .. } => {
            let w = stroke.map_or(0.0, |(w, _)| w);
            Shape::Path {
                cmds: cmds.clone(),
                rule: Default::default(),
            }
            .bounds()
            .inflate(w, w)
        }
        DrawCommand::Shadow { rect, std_dev, .. } => {
            let r = SHADOW_REACH * std_dev;
            rect.inflate(r, r)
        }
        DrawCommand::Glyphs { run, origin, .. } => {
            let mut bounds = glyphs(&run.glyphs, run.font_size)?;
            bounds.x += origin.x;
            bounds.y += origin.y;
            bounds
        }
        DrawCommand::FillShape { shape, .. }
        | DrawCommand::Backdrop(kivact_types::protocol::Backdrop { shape, .. }) => shape.bounds(),
        DrawCommand::StrokeShape { shape, stroke, .. } => {
            let joints = if stroke.join == LineJoin::Miter {
                stroke.miter_limit.max(1.5)
            } else {
                1.5
            };
            let w = stroke.width * joints;
            shape.bounds().inflate(w, w)
        }
        DrawCommand::BoxShadow(shadow) => box_shadow(shadow),
        DrawCommand::PushClip(_)
        | DrawCommand::PopClip
        | DrawCommand::PushTransform(_)
        | DrawCommand::PopTransform
        | DrawCommand::PushLayer(_)
        | DrawCommand::PopLayer => return None,
    })
}

fn points(points: &[kivact_types::Point]) -> Rect {
    Shape::Polygon {
        points: points.to_vec(),
        rule: Default::default(),
    }
    .bounds()
}

/// Glyphs' ink, generously: an em around each origin and its advance.
fn glyphs(glyphs: &[Glyph], size: f32) -> Option<Rect> {
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
    Some(Rect::new(
        b.0 - size,
        b.1 - 1.5 * size,
        b.2 - b.0 + 2.0 * size,
        b.3 - b.1 + 2.5 * size,
    ))
}

fn box_shadow(shadow: &BoxShadow) -> Rect {
    let rect = shadow.shape.rect;
    if shadow.inset {
        return rect;
    }
    let r = SHADOW_REACH * shadow.std_dev + shadow.spread.max(0.0);
    let cast = Rect::new(
        rect.x + shadow.offset.x,
        rect.y + shadow.offset.y,
        rect.width,
        rect.height,
    )
    .inflate(r, r);
    rect.union(cast)
}
