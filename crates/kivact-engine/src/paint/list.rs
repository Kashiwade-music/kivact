//! Role: The retained, culled paint list of the nodes a frame draws in paint order.
//!
//! The retained paint list: the nodes a frame draws, in paint order (the
//! tree's, children by z-index), with the origin of their untransformed
//! layout box, the transform they are drawn through and the opacity they
//! are drawn at, culled against the window and their clipping ancestors.
//!
//! Invariants:
//! - The list is rebuilt only when something may move, show, hide,
//!   restack or reclip nodes (typed setter invalidation for writes, `Model::sync`
//!   for layout passes, scrolls and state styles), so a frame after a
//!   content change (text, values, colors, images) draws the same nodes
//!   again with their current content: O(visible nodes), not O(nodes). A
//!   write of a paint, a text style or state styles asks for a rebuild
//!   only when what the steps are made of changed: the node's `Place`
//!   (transform, z-index, sticky, fixed), its paint's [`Look::stacking`]
//!   facts (opacity, group, items, rounding, overflow,
//!   border, filter reach, clip) or, for a text, how far it paints out of
//!   its box.
//! - A node is left out when everything it can draw (its reach) lies
//!   outside the clip its ancestors impose, grown by [`MARGIN`](walk::MARGIN) (so a
//!   pixel the clip lets through partially is never lost): its box, the
//!   extent of its content (taffy's, text overflowing a short box
//!   included), its shadows (an outer one reaches its offset plus its
//!   spread plus `kivact_types::render::SHADOW_REACH` × blur / 2) and focus ring, a
//!   text's glyph overhang (one font size) and text shadows, and its
//!   canvas scenes' commands (a scene text run counted at two font sizes
//!   and its letter spacing per character, a stroke at its width × its
//!   miter limit, each mapped by the scene's own transforms and grown by
//!   its layers' filters), all mapped by its transform.
//! - Its subtree is left out with it only when the subtree's ink (its
//!   reach and, unless it clips, its children's inks at their locations,
//!   all mapped by its transform: an absolutely placed badge above it, a
//!   child's wide shadow, a child transformed out of it) lies outside the
//!   clip too. Culling never changes a pixel of the frame.
//! - A transformed node is drawn through its transform (pushed around its
//!   whole drawing, its subtree included), and its subtree is culled
//!   against the clip mapped back into its own space (the bounds of the
//!   inverse); a transform that flattens it lists nothing. A translation
//!   alone moves the origin of the node and its subtree instead (no push:
//!   the same pixels, less work for the many moving boxes of an
//!   animation), so the origins are in the own space of the nearest pushed
//!   transform moved by the translations below it.
//! - The first build computes the ink of every subtree it reaches (so
//!   culling never drops a descendant drawn outside its ancestor), and a
//!   node keeps its ink ([`Ink`]) until something in its subtree changes
//!   (a write, a repaint, a scroll, a new measure: `PaintList::invalidate`
//!   and `PaintList::forget_ink` forget it and its ancestors'), its own
//!   local rounded geometry changes (ancestor placement can change a child's
//!   pixel rounding even when the ancestor's size stays the same), or the
//!   generation moves on (new fonts).
//!   Position-only changes keep the node's local ink and invalidate its
//!   parent's union; the layout pass refreshes child-index locations.
//! - Children follow the tree's order, sorted (stably) by
//!   `Model::stack_key` (the z-index, then sticky after its siblings of the
//!   same z-index) only in a container whose `Place::stacks` is set (a
//!   child with a z-index, or a state style that can give it one, set it
//!   once for good); any other container costs nothing for it.
//! - The layers (`Model::layers`: the fixed boxes outside every overlay,
//!   then each overlay in attach order followed by the fixed boxes inside
//!   it) each start afresh after the root's tree at their window origin,
//!   with the window as their clip and full opacity: they
//!   take none of their ancestors' scrolls, clips, transforms or opacity, and are not in their parent's ink or child index.
//! - A node is composited in a group of its own only when it needs one: a
//!   clip path or a blend mode, or an opacity
//!   below 1 over more than one thing it draws (children, a scene, or two
//!   of its background, border, shadows, ring and content), which is CSS's
//!   group opacity; a lone background or text is faded by its colors'
//!   alpha instead. Such a node is listed (to open and close its group)
//!   even when only its descendants show. Its subtree is drawn inside
//!   them at full alpha.
//! - A layer filter (blur, drop shadow) grows the node's reach and ink by
//!   how far it draws, and the clip its children are culled against, so a
//!   blurred pixel from outside the window still reaches it; a backdrop
//!   filter grows the whole viewport by its blur's reach (the largest
//!   seen), as it reads what lies around its box.
//! - A rebuild is proportional to the nodes it lists, not to the tree: a
//!   container with many children keeps a child index ([`ChildIndex`]):
//!   each child's ink (in the container's unscrolled content space) sorted
//!   along the axis the children follow, so a rebuild finds the children
//!   that reach the clip by binary search. After a layout pass an index
//!   takes the children's new locations, looking again only at the
//!   children whose size changed; a change in a child's subtree makes it
//!   look at that child again; a change of the container's children drops
//!   it (`PaintList::invalidate`). Scrolling keeps it.

mod index;
mod ink;
mod invalidation;
mod layers;
mod look;
#[cfg(all(debug_assertions, feature = "testing"))]
mod self_check;
mod walk;
