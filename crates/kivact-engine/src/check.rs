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

mod boxes;
mod findings;
#[cfg(all(debug_assertions, feature = "testing"))]
mod layout_reference;
mod shaped_lines;

/// Whether the node's native text shaping cut any displayed line.
pub(crate) fn text_truncated(
    host: &crate::model::Model,
    engine: &mut arvane::Engine,
    id: kivact_wire::NodeId,
) -> bool {
    if !host
        .node(id)
        .is_some_and(|node| node.kind() == kivact_wire::NodeType::Text)
    {
        return false;
    }
    let props = host.text_props(id);
    let more = props.more();
    if more.max_lines.is_none() && more.overflow != kivact_types::TextOverflow::Ellipsis {
        return false;
    }
    shaped_lines::shape(host, engine, id, false)
        .is_some_and(|text| text.lines.iter().any(|line| line.cut))
}

#[cfg(debug_assertions)]
pub(crate) use boxes::visual_parent_of;
#[cfg(feature = "testing")]
pub(crate) use findings::layout_findings;
#[cfg(debug_assertions)]
pub(crate) use findings::text_findings;
