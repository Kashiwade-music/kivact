//! Role: Invalidates a node's layout and its parent chain on style or content change.

use crate::calcs::CalcAlloc;
use crate::style::node::NodeStyle;
use crate::{Flow, LayoutState, LayoutTree, NodeLayoutColumns};
use kivact_types::NodeId;
use taffy::ClearState;

/// Whether `raw` is a percentage or a calc (taffy 0.15 keeps a tag in the low
/// bits of a calc handle, so a calc is found by `is_plain_calc`, not its tag).
fn relative_length(raw: taffy::style::CompactLength) -> bool {
    raw.tag() == taffy::style::CompactLength::PERCENT_TAG || raw.is_plain_calc()
}

/// Whether `s` resolves a length of its own box against its parent's size.
fn uses_parent_size(s: &NodeStyle) -> bool {
    let d = |v: taffy::Dimension| relative_length(v.into_raw());
    let a = |v: taffy::LengthPercentageAuto| relative_length(v.into_raw());
    let p = |v: taffy::LengthPercentage| relative_length(v.into_raw());
    d(s.size.width)
        || d(s.size.height)
        || d(s.flex_basis)
        || a(s.min_size.width)
        || a(s.min_size.height)
        || a(s.max_size.width)
        || a(s.max_size.height)
        || [
            s.margin.left,
            s.margin.right,
            s.margin.top,
            s.margin.bottom,
            s.inset.left,
            s.inset.right,
            s.inset.top,
            s.inset.bottom,
        ]
        .into_iter()
        .any(a)
        || [
            s.padding.left,
            s.padding.right,
            s.padding.top,
            s.padding.bottom,
            s.border.left,
            s.border.right,
            s.border.top,
            s.border.bottom,
        ]
        .into_iter()
        .any(p)
}

impl NodeLayoutColumns {
    /// Replace compiled style from authoritative raw props and invalidate its chain.
    /// Tree, state and columns must share the same slot allocation domain.
    /// An absent identity is ignored; every resolved slot must already be allocated.
    pub(crate) fn rebuild_style<Tree: LayoutTree + ?Sized>(
        &mut self,
        tree: &Tree,
        state: &mut LayoutState,
        id: NodeId,
    ) {
        let Some(slot) = tree.index_of(id) else {
            return;
        };
        let index = slot + 1;
        let style = {
            self.calcs
                .free
                .extend(std::mem::take(&mut self.states[index].calcs));
            let mut alloc = CalcAlloc {
                calcs: &mut self.calcs,
                owned: Vec::new(),
            };
            let (props, hidden) = tree.layout_props(slot);
            let style =
                crate::style::taffy_style(props.unwrap_or(&Default::default()), hidden, &mut alloc);
            self.states[index].calcs = alloc.owned;
            style
        };
        let root = self.values.styles[index].position == taffy::Position::Absolute
            && style.position == taffy::Position::Absolute;
        self.values.relative[index] = uses_parent_size(&style);
        self.values.styles[index] = self.values.recent_styles.share(style);
        self.values.grid_infos.remove(&index);
        state.mark_dirty();
        self.states[index].measured = Default::default();
        if root {
            self.clear_caches_up(tree, state, slot);
        } else {
            self.clear_caches_through(tree, state, slot);
        }
    }
    /// Invalidate a content measurement and its dependency chain.
    /// Tree, state and columns must share the same slot allocation domain.
    /// Absent node identities mark work changed without accessing a slot.
    pub(crate) fn invalidate_chain<Tree: LayoutTree + ?Sized>(
        &mut self,
        tree: &Tree,
        state: &mut LayoutState,
        id: NodeId,
    ) {
        state.mark_dirty();
        if let Some(slot) = tree.index_of(id) {
            self.states[slot + 1].measured = Default::default();
            self.clear_caches_up(tree, state, slot);
        }
    }
    /// Invalidate the shared viewport's layout (the container of the fixed boxes).
    pub(crate) fn invalidate_viewport(&mut self, state: &mut LayoutState) {
        state.mark_dirty();
        if let Some(cache) = self.values.caches[0].as_mut() {
            cache.clear();
        }
    }
    pub(super) fn clear_caches_up<Tree: LayoutTree + ?Sized>(
        &mut self,
        tree: &Tree,
        state: &mut LayoutState,
        slot: usize,
    ) {
        let mut at = Some(slot);
        while let Some(slot) = at {
            // Contents boxes never participate in layout, so their empty
            // caches cannot prove that the contributing parent is clean.
            if tree
                .layout_props(slot)
                .0
                .is_some_and(|props| props.display == kivact_types::Display::Contents)
            {
                at = tree.parent(slot);
                continue;
            }
            let flow = tree.flow(slot);
            let parent = tree.parent(slot);
            let values = &mut self.values;
            if let ClearState::AlreadyEmpty = values.caches[slot + 1]
                .as_mut()
                .map_or(ClearState::AlreadyEmpty, |cache| cache.clear())
            {
                break;
            }
            if flow == Flow::Overlay {
                break;
            }
            if flow == Flow::Fixed && parent.is_some() {
                if let Some(cache) = values.caches[0].as_mut() {
                    cache.clear();
                }
                break;
            }
            if values.styles[slot + 1].position == taffy::Position::Absolute
                && let Some(parent) = parent
            {
                state.queue_relayout(tree.id_at(parent).expect("invariant: layout parent exists"));
                break;
            }
            at = parent;
        }
    }
    pub(crate) fn clear_caches_through<Tree: LayoutTree + ?Sized>(
        &mut self,
        tree: &Tree,
        state: &mut LayoutState,
        slot: usize,
    ) {
        let flow = tree.flow(slot);
        let parent = tree.parent(slot);
        if let Some(cache) = self.values.caches[slot + 1].as_mut() {
            cache.clear();
        }
        if flow == Flow::Overlay {
            return;
        }
        if flow == Flow::Fixed && parent.is_some() {
            if let Some(cache) = self.values.caches[0].as_mut() {
                cache.clear();
            }
        } else if let Some(parent) = parent {
            self.clear_caches_up(tree, state, parent);
        }
    }
}
