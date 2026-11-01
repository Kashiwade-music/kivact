//! Role: Layout check that finds texts laid out wrong from shapes the engine has.
//!
//! The layout check: finds texts that are laid out wrong, from the shapes
//! the engine already has (the lines of a shaped text, their advances and
//! metrics, the ink of their glyphs) and the boxes of the layout. The test
//! driver runs it on demand (`layout_findings`, with the ink, for
//! `ArvaneTestHost::layout_findings`); in debug builds the engine runs it
//! after a layout pass for the texts whose boxes changed, without the ink
//! (`lint`). It walks the `Text` nodes and native editing leaves in tree
//! order and gives one `LayoutFinding` per defect (the node, the rule, how
//! far, the text's rect and the box it broke).
//!
//! A text is shaped as it is painted (`shape_glyphs` at its box's width
//! plus 1 px, colors aside), so the check shares the paint's shaping cache.
//! A line's rect is its advance across (`TextLine::rect`: what an ellipsis
//! left of it, the ellipsis included) and its ink down, the glyphs'
//! outlines (`arvane::glyph_ink_bounds`), else from its capitals' top (its
//! ascent without a cap height) to its descent. A multiline text input
//! (whose lines scroll) is not checked.
//!
//! The rules (`LayoutRule`), with the defaults of `LayoutCheck::default`:
//! - **Overflow.** The text's rect (a line's advance across, its ink
//!   down) reaches outside its own content box, or the padding box of its
//!   visual parent, by more than the tolerance (`overflow_tolerance`,
//!   1 px); the amount is the most of its four sides. A scrolling axis,
//!   and the horizontal axis of a text input (which scrolls), cannot
//!   overflow.
//! - **Wrapped.** A text with two lines or more below (or with) a role that
//!   expects one line (`single_line_roles`: Button, Tab, MenuItem,
//!   MenuItemCheckBox, MenuItemRadio, Option, ComboBox, Switch, Link,
//!   ColumnHeader), when its content has no line feed and it is not
//!   clamped to two lines or more; the amount is the height of the lines
//!   after the first.
//! - **OffCenter.** A single line whose boxes up to the visual parent all
//!   center it vertically is not in the middle of the parent (the middle
//!   of a line is half a capital above its baseline; the tolerance,
//!   `center_tolerance`, is `max(1, 0.08 × font size)` by default, and gets
//!   half a pixel on top for the whole-pixel layout). A box centers its
//!   child when it is a row without wrapping that aligns the child (by its
//!   own `align_self` first) to the center, or a column that justifies to
//!   the center with no other child in flow; a grid, and a child out of
//!   flow, center nothing. A line with no cap height is not checked.
//! - **ClippedWithoutEllipsis.** A clipping ancestor cuts the ink of a
//!   line that no ellipsis cut, across an axis it does not scroll (a text
//!   input's horizontal one aside); the nearest such ancestor is
//!   reported.
//!
//! Invariants:
//! - Everything is compared in the layout space (the window's, before any
//!   transform), where the boxes and the text are placed alike.
//! - The visual parent is the nearest ancestor with a background that
//!   draws (not transparent), a border, a clip or a single-line role. The
//!   walk up ends at an overlay or a fixed box (the last ancestor looked
//!   at): the boxes above are not where it is placed.
//! - A text a clip cuts is reported as `ClippedWithoutEllipsis` against
//!   that box, not also as an overflow of it; a text that overflows its
//!   visual parent by no more than it overflows its own box (with the
//!   tolerance) is reported against its own box only.
//! - A hidden node, and the subtree of an ignored test id, are not looked
//!   at. The check changes nothing and sends no event (`lint` does).

#[cfg(debug_assertions)]
pub(crate) mod lint;

use crate::model::Model;
use crate::model::interact::visual;
use crate::model::semantics::semantic_role;
use crate::tree::node::HostNode;
use kivact_wire::{
    Align, Edges, Fill, FillImage, FlexDirection, Justify, LayoutCheck, LayoutFinding, LayoutLimit,
    LayoutProps, LayoutRule, NodeId, NodeType, Point, Position, Rect,
};

/// How far (px) the whole-pixel layout can move a box from its ideal
/// place.
const PIXEL_ROUNDING: f32 = 0.5;

/// The findings of the texts in the subtree of `scope` (the whole tree
/// without one), in tree order; `ink`: the vertical extent of a line is
/// that of its glyphs' outlines, else of its metrics.
#[cfg(feature = "testing")]
pub(crate) fn layout_findings(
    host: &Model,
    engine: &mut arvane::Engine,
    check: &LayoutCheck,
    scope: Option<NodeId>,
    ink: bool,
) -> Vec<LayoutFinding> {
    let mut out = Vec::new();
    let Some(start) = scope.or(host.tree().root()) else {
        return out;
    };
    if !host.tree().shown(start) || ignored_above(host, start, check) {
        return out;
    }
    let mut stack = vec![start];
    while let Some(id) = stack.pop() {
        let Some(n) = host.node(id) else { continue };
        if n.hidden() || ignores(n, check) {
            continue;
        }
        if n.kind() == NodeType::Text
            || host.editors[n.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())
                .is_some()
        {
            text_findings(host, engine, check, id, ink, &mut out);
        }
        stack.extend(host.tree().children_ids(n).rev());
    }
    out
}

/// Whether `id` or an ancestor has a test id the check ignores.
#[cfg(feature = "testing")]
fn ignored_above(host: &Model, id: NodeId, check: &LayoutCheck) -> bool {
    !check.ignore_test_ids.is_empty()
        && host
            .tree()
            .chain(Some(id))
            .into_iter()
            .any(|a| host.node(a).is_some_and(|n| ignores(n, check)))
}

#[cfg(feature = "testing")]
fn ignores(n: &HostNode, check: &LayoutCheck) -> bool {
    n.semantics()
        .and_then(|s| s.test_id.as_ref())
        .is_some_and(|t| check.ignore_test_ids.iter().any(|i| i == t))
}

/// The ancestors of `id`, nearest first, up to and including the first
/// overlay or fixed box (none for such a box itself).
fn ancestors(host: &Model, id: NodeId) -> Vec<NodeId> {
    let tree = host.tree();
    let mut out = Vec::new();
    let Some(n) = tree.get(id) else { return out };
    if n.is_layer() {
        return out;
    }
    let mut at = tree.parent_id(n);
    while let Some(p) = at {
        let Some(pn) = tree.get(p) else { break };
        out.push(p);
        if pn.is_layer() {
            break;
        }
        at = tree.parent_id(pn);
    }
    out
}

/// The nearest of `ancestors` that paints a box of its own or expects one
/// line: a background that draws, a border, a clip or a single-line role.
fn visual_parent(host: &Model, ancestors: &[NodeId], check: &LayoutCheck) -> Option<NodeId> {
    ancestors.iter().copied().find(|a| {
        host.node(*a).is_some_and(|n| {
            n.paint().is_some_and(|p| {
                p.background.as_ref().is_some_and(paints)
                    || kivact_lower::shape::border_widths(p.border.as_ref()) != Edges::all(0.0)
            }) || n.clips()
                || semantic_role(n, host.editors[n.index.0].is_some())
                    .is_some_and(|r| check.single_line_roles.contains(&r))
        })
    })
}

/// The visual parent of the text `id`.
#[cfg(debug_assertions)]
pub(crate) fn visual_parent_of(host: &Model, id: NodeId, check: &LayoutCheck) -> Option<NodeId> {
    visual_parent(host, &ancestors(host, id), check)
}

/// Whether `fill` draws anything.
fn paints(fill: &Fill) -> bool {
    match fill {
        Fill::Solid(c) => c.a > 0.0,
        Fill::Layers(layers) => layers.iter().any(|l| match &l.image {
            FillImage::Solid(c) => c.a > 0.0,
            _ => true,
        }),
    }
}

/// A shaped line, placed in the layout space.
struct Line {
    /// Its ink: the advance across, the glyphs' outlines (or, without
    /// them, the capitals' top to the descent) down.
    ink: Rect,
    /// Its baseline.
    baseline: f32,
    /// The height of the capitals above the baseline.
    cap_height: f32,
    /// The height of its line box.
    height: f32,
    /// An ellipsis cut it.
    cut: bool,
}

/// A text shaped as it is painted, with its lines in the layout space.
struct Shaped {
    lines: Vec<Line>,
    font_size: f32,
}

impl Shaped {
    /// The rect the text covers: its lines' advances across, their ink
    /// down.
    fn rect(&self) -> Rect {
        self.lines
            .iter()
            .map(|l| l.ink)
            .reduce(|a, b| a.union(b))
            .unwrap_or_default()
    }
}

/// Shape the text `id` as it is painted (`None`: it draws nothing, or is
/// not checked: a multiline input scrolls its lines).
fn shape(host: &Model, engine: &mut arvane::Engine, id: NodeId, ink: bool) -> Option<Shaped> {
    let n = host.node(id)?;
    let props = host.text_props(id);
    debug_assert_eq!(
        *props,
        crate::measure::text_props(host.tree(), id),
        "node {} keeps a stale computed text style",
        id.0
    );
    let area = visual::text_box(host, id);
    let (shaped, origin) = match n.kind() {
        _ if host.editors[n.index.0]
            .as_deref()
            .map(|editor| editor.editor.state())
            .is_some()
            && !n.multiline() =>
        {
            let line = crate::model::interact::text_geometry::line_box(host, id, engine, area);
            let (mut lines, text) = host.edit_lines(id, engine)?;
            (
                lines.shaped(text, 0, engine)?,
                Point {
                    x: area.x + lines.offset(text, 0, area.width, engine),
                    y: line.y,
                },
            )
        }
        NodeType::Text
            if host.editors[n.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())
                .is_none() =>
        {
            let mut style = kivact_lower::assets::text_style(engine, &props);
            // As the paint shapes it: colors never change a shape.
            style.color = kivact_types::TextStyle::default().color;
            style.decoration.color = None;
            let width = Some(area.width.max(0.0) + 1.0);
            let content = n.content().unwrap_or("");
            let shaped = if n.spans().is_empty() {
                engine.shape_glyphs(content, &style, width)
            } else {
                let spans = crate::measure::leaf::rich_spans(engine, n.spans(), &style);
                let text = kivact_types::RichText {
                    text: content,
                    spans: &spans,
                };
                engine.shape_glyphs(text, &style, width)
            };
            (
                shaped.ok()?,
                Point {
                    x: area.x,
                    y: area.y,
                },
            )
        }
        _ => return None,
    };
    let mut inks: Vec<Option<Rect>> = vec![None; shaped.lines.len()];
    if ink {
        for run in &shaped.runs {
            let (Some(b), Some(g)) = (arvane::glyph_ink_bounds(run), run.glyphs.first()) else {
                continue;
            };
            // The line whose baseline this run sits on.
            let nearest = (0..shaped.lines.len()).min_by(|a, b| {
                let d = |i: usize| (shaped.lines[i].baseline - g.y).abs();
                d(*a).total_cmp(&d(*b))
            });
            if let Some(i) = nearest {
                let b = Rect {
                    x: origin.x + b.x,
                    y: origin.y + b.y,
                    width: b.width,
                    height: b.height,
                };
                inks[i] = Some(inks[i].map_or(b, |a| a.union(b)));
            }
        }
    }
    let lines = shaped
        .lines
        .iter()
        .zip(inks)
        .map(|(l, ink)| {
            let x = origin.x + l.rect.x;
            // From the capitals' top to the descent: where a line's ink
            // lies without looking at the glyphs.
            let top = if l.cap_height > 0.0 {
                l.cap_height
            } else {
                l.ascent
            };
            let metrics = Rect {
                x,
                y: origin.y + l.baseline - top,
                width: l.rect.width,
                height: top + l.descent,
            };
            // The advance across, the ink down.
            let ink = ink.map_or(metrics, |i| Rect {
                x,
                width: l.rect.width,
                ..i
            });
            Line {
                ink,
                baseline: origin.y + l.baseline,
                cap_height: l.cap_height,
                height: l.rect.height,
                cut: l.cut,
            }
        })
        .collect();
    Some(Shaped {
        lines,
        font_size: props.font_size,
    })
}

/// The axes a limit holds on.
#[derive(Clone, Copy)]
struct Axes {
    x: bool,
    y: bool,
}

impl Axes {
    /// Every axis but those `n` scrolls (a text input scrolls its text
    /// across).
    fn held_by(n: &HostNode, input: bool) -> Axes {
        let (sx, sy) = n.scroll_axes();
        Axes {
            x: !sx && !input,
            y: !sy,
        }
    }
}

/// How far `r` reaches outside `limit` across `axes` (at most 0 inside).
fn reach(r: Rect, limit: Rect, axes: Axes) -> f32 {
    let mut most = 0.0f32;
    if axes.x {
        most = most
            .max(limit.x - r.x)
            .max(r.x + r.width - (limit.x + limit.width));
    }
    if axes.y {
        most = most
            .max(limit.y - r.y)
            .max(r.y + r.height - (limit.y + limit.height));
    }
    most
}

/// The padding box of `id` (its border box inside its visible border), in
/// the layout space.
fn padding_box(host: &Model, id: NodeId) -> Option<Rect> {
    let bounds = host.rect(id)?;
    let paint = host.node(id)?.paint().cloned().unwrap_or_default();
    Some(visual::padding_box(bounds, &paint))
}

/// A box that cuts what it holds: which node it is, where, and across
/// which axes.
struct Clip {
    node: NodeId,
    rect: Rect,
    axes: Axes,
}

/// The clipping ancestors of the text `id` (`up`, nearest first), with the
/// padding box each clips to.
fn clips(host: &Model, id: NodeId, up: &[NodeId]) -> Vec<Clip> {
    let input = host.node(id).is_some_and(|n| {
        host.editors[n.index.0]
            .as_deref()
            .map(|editor| editor.editor.state())
            .is_some()
    });
    up.iter()
        .filter_map(|a| {
            let n = host.node(*a).filter(|n| n.clips())?;
            Some(Clip {
                node: *a,
                rect: padding_box(host, *a)?,
                axes: Axes::held_by(n, input),
            })
        })
        .collect()
}

/// The findings of the text or text input `id`, appended to `out`.
pub(crate) fn text_findings(
    host: &Model,
    engine: &mut arvane::Engine,
    check: &LayoutCheck,
    id: NodeId,
    ink: bool,
    out: &mut Vec<LayoutFinding>,
) {
    let Some(n) = host.node(id) else { return };
    let Some(shaped) = shape(host, engine, id, ink) else {
        return;
    };
    if shaped.lines.is_empty() {
        return;
    }
    let input = host.editors[n.index.0]
        .as_deref()
        .map(|editor| editor.editor.state())
        .is_some();
    let tolerance = check.overflow_tolerance;
    let text = shaped.rect();
    let up = ancestors(host, id);
    let vp = visual_parent(host, &up, check);
    let mut push = |rule, amount, text_rect, limit_rect| {
        out.push(LayoutFinding {
            node: id,
            rule,
            amount,
            text_rect,
            limit_rect,
        });
    };

    // Clipped with no ellipsis: by the nearest box that cuts a line.
    let mut clipped_at = None;
    for clip in clips(host, id, &up) {
        let cut: Vec<(f32, Rect)> = shaped
            .lines
            .iter()
            .filter(|l| !l.cut)
            .map(|l| (reach(l.ink, clip.rect, clip.axes), l.ink))
            .filter(|(by, _)| *by > tolerance)
            .collect();
        if let Some(most) = cut.iter().map(|(by, _)| *by).reduce(f32::max) {
            let rect = cut
                .iter()
                .map(|(_, r)| *r)
                .reduce(|a, b| a.union(b))
                .unwrap_or(text);
            push(LayoutRule::ClippedWithoutEllipsis, most, rect, clip.rect);
            clipped_at = Some(clip.node);
            break;
        }
    }

    // Overflow: out of its own box, else out of its visual parent's.
    let own = visual::text_box(host, id);
    let own_by = reach(text, own, Axes::held_by(n, input));
    if own_by > tolerance {
        push(
            LayoutRule::Overflow {
                into: LayoutLimit::OwnBox,
            },
            own_by,
            text,
            own,
        );
    }
    if let Some(v) = vp.filter(|v| clipped_at != Some(*v))
        && let (Some(vn), Some(limit)) = (host.node(v), padding_box(host, v))
    {
        let by = reach(text, limit, Axes::held_by(vn, input));
        if by > tolerance && by > own_by + tolerance {
            push(
                LayoutRule::Overflow {
                    into: LayoutLimit::VisualParent,
                },
                by,
                text,
                limit,
            );
        }
    }

    // Wrapped: more than one line below a role that expects one.
    let lines = shaped.lines.len();
    if lines >= 2 && !n.content().is_some_and(|c| c.contains('\n')) {
        let clamped = host.text_props(id).more().max_lines.is_some_and(|m| m >= 2);
        let owner = std::iter::once(id)
            .chain(up.iter().copied())
            .find(|a| single_line(host, *a, check));
        if let (false, Some(owner)) = (clamped, owner)
            && let Some(limit) = host.rect(owner)
        {
            let extra = shaped.lines[1..].iter().map(|l| l.height).sum();
            push(
                LayoutRule::Wrapped {
                    lines: lines as u32,
                },
                extra,
                text,
                limit,
            );
        }
    }

    // Off center: every box up to the visual parent centers it.
    if let (1, Some(v)) = (lines, vp) {
        let upto = up.iter().position(|a| *a == v).map_or(0, |i| i + 1);
        let line = &shaped.lines[0];
        if line.cap_height > 0.0 && centered_in(host, id, &up[..upto]) {
            let content = visual::text_box(host, v);
            let middle = line.baseline - line.cap_height / 2.0;
            let off = (middle - (content.y + content.height / 2.0)).abs();
            // The layout rounds boxes to whole pixels: half a pixel of
            // slack on top, or an odd leftover would always be off.
            let allowed = check
                .center_tolerance
                .unwrap_or_else(|| (shaped.font_size * 0.08).max(1.0))
                + PIXEL_ROUNDING;
            if off > allowed {
                push(LayoutRule::OffCenter, off, text, content);
            }
        }
    }
}

/// Whether the role of `id` expects its text on one line.
fn single_line(host: &Model, id: NodeId, check: &LayoutCheck) -> bool {
    host.node(id)
        .and_then(|node| semantic_role(node, host.editors[node.index.0].is_some()))
        .is_some_and(|r| check.single_line_roles.contains(&r))
}

/// Whether each of `boxes` (the ancestors of `child`, nearest first)
/// centers its child vertically, so the child sits in the middle of the
/// last.
fn centered_in(host: &Model, child: NodeId, boxes: &[NodeId]) -> bool {
    let mut child = child;
    for b in boxes {
        let Some(parent) = host.node(*b) else {
            return false;
        };
        if !centers(host, parent, child) {
            return false;
        }
        child = *b;
    }
    true
}

/// Whether `parent` centers `child` vertically: a row aligning it in the
/// middle, or a column justifying its only in-flow child there.
fn centers(host: &Model, parent: &HostNode, child: NodeId) -> bool {
    let default = LayoutProps::default();
    let layout = parent.layout().unwrap_or(&default);
    if layout.wrap || layout.grid.as_ref().is_some_and(|g| g.container.is_some()) {
        return false;
    }
    // A box out of the flow sits at its insets, wherever its parent aligns
    // the others.
    let position = host
        .node(child)
        .and_then(|c| c.layout())
        .map(|l| l.position);
    if !matches!(position, None | Some(Position::Relative | Position::Sticky)) {
        return false;
    }
    match layout.direction {
        FlexDirection::Row | FlexDirection::RowReverse => {
            let own = host
                .node(child)
                .and_then(|c| c.layout())
                .and_then(|l| l.align_self);
            own.or(layout.align_items) == Some(Align::Center)
        }
        FlexDirection::Column | FlexDirection::ColumnReverse => {
            layout.justify == Some(Justify::Center)
                && host
                    .tree()
                    .children_ids(parent)
                    .filter(|c| *c != child)
                    .filter_map(|c| host.node(c))
                    .all(|c| {
                        c.hidden()
                            || c.is_layer()
                            || c.layout().is_some_and(|l| l.position == Position::Absolute)
                    })
        }
    }
}

#[cfg(all(debug_assertions, feature = "testing"))]
mod layout_reference;
