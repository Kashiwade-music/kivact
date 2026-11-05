//! Role: Build one node's accessibility payload under the full builder's policy.

use crate::model::Model;
use crate::{is_list_role, semantic_role};
use arvane as a;
use kivact_types as types;
use std::collections::{BTreeMap, BTreeSet};

impl Model {
    /// The focus the tree reports: the focused node when it is exposed,
    /// shown and focusable.
    pub(super) fn semantics_focus(
        &self,
        clipped: &BTreeSet<types::NodeId>,
    ) -> Option<kivact_types::SemanticsId> {
        let f = self.focused()?;
        self.exposed(f.node)?;
        if self.hidden(f.node, clipped) || !self.is_focus_target(f) || !self.in_scope(f.node) {
            return None;
        }
        match f.region {
            Some(region) => self
                .a11y
                .region_ids
                .get(&(f.node, region))
                .copied()
                .map(kivact_types::SemanticsId),
            None => Some(kivact_types::SemanticsId(f.node.0)),
        }
    }

    /// Attached to the root (an overlay through its parent).
    pub(super) fn attached(&self, id: types::NodeId) -> bool {
        self.tree().chain(Some(id)).last().copied() == self.tree().root()
    }

    /// The scroll offsets of the ancestors of `id`, summed.
    pub(super) fn scroll_above(&self, id: types::NodeId) -> types::Point {
        let mut out = types::Point::default();
        // From the root down, summed in the order a whole build sums them.
        for x in self.tree().chain(Some(id)).into_iter().skip(1).rev() {
            let s = self.node(x).map(|n| n.scroll()).unwrap_or_default();
            out.x += s.x;
            out.y += s.y;
        }
        out
    }

    fn hidden(&self, id: types::NodeId, clipped: &BTreeSet<types::NodeId>) -> bool {
        !self.tree().shown(id) || clipped.contains(&id)
    }

    fn focusable(&self, id: types::NodeId, hidden: bool) -> bool {
        !hidden && self.is_tab_stop(id) && self.in_scope(id)
    }

    /// Build node `id` (and its scene regions' leaves) into `out`;
    /// `offset` is the scroll above it. The native ids of its regions go
    /// into `regions`.
    pub(super) fn build_node(
        &mut self,
        id: types::NodeId,
        offset: types::Point,
        clipped: &BTreeSet<types::NodeId>,
        out: &mut Vec<kivact_types::render::SemanticsNode>,
        regions: &mut BTreeSet<u64>,
        engine: &mut a::Engine,
    ) {
        let Some(n) = self.node(id) else {
            return;
        };
        let hidden = self.hidden(id, clipped);
        let rect = self.rect(id).unwrap_or_default();
        let bounds = types::Rect {
            x: rect.x + offset.x,
            y: rect.y + offset.y,
            ..rect
        };
        let scroll = n.scroll();

        let transform = if n.place.transformed || n.is_layer() {
            self.a11y_transform(id, n.is_layer(), bounds, offset)
        } else {
            kivact_types::Transform::IDENTITY
        };
        let children: Vec<kivact_types::SemanticsId> = self
            .tree()
            .flat_children(self.tree().nodes.id_at(n.index))
            .map(|id| kivact_types::SemanticsId(id.0))
            .collect();
        // The model's exposure (role, name, actions). A node without a
        // role is not exposed: it stays a bare structural node (a
        // generic container, which AccessKit's platform adapters leave
        // out while lifting its children to the parent) without a name,
        // state, actions, focus or scene regions.
        let Some(exposed) = self.exposed(id) else {
            out.push(structural(id, children, bounds, scroll, transform, hidden));
            return;
        };
        let disabled = n.disabled();
        let focusable = self.focusable(id, hidden);
        let actions = focus_actions(hidden, focusable, exposed.actions);
        let scene_regions = scene_regions(n);
        let extras = self.node_properties(id, n);
        let authored_text = n.semantics().and_then(|props| props.text.clone());
        let mut output = kivact_types::render::SemanticsNode {
            id: kivact_types::SemanticsId(id.0),
            children,
            bounds,
            scroll,
            transform,
            semantics: kivact_types::Semantics {
                role: Some(exposed.role),
                name: Some(exposed.name),
                description: Some(
                    n.semantics()
                        .and_then(|s| s.description.clone())
                        .unwrap_or_default(),
                ),
                value_text: match self.editors[n.index.0]
                    .as_deref()
                    .map(|editor| editor.editor.state())
                {
                    Some(_) => None,
                    None => n.semantics().and_then(|s| s.value_text.clone()),
                },
                disabled,
                hidden,
                focusable,
                checked: n.semantics().and_then(|s| s.checked),
                selected: Some(n.selected()),
                expanded: n.semantics().and_then(|s| s.expanded),
                test_id: n.semantics().and_then(|s| s.test_id.clone()),
                actions: Some(actions),
                ..*extras.unwrap_or_default()
            },
        };
        if self.node(id).is_some_and(|n| {
            self.editors[n.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())
                .is_some()
                && !n.password()
        }) {
            let text = self.editing_semantics(id, offset, engine);
            output.semantics.text = Some(text);
        } else if let Some(authored) = authored_text {
            // A node that draws and edits its own text publishes its own runs.
            let text = self.authored_semantics(id, &authored, bounds);
            output.semantics.text = text;
        }
        let leaves = self.build_region_leaves(id, scene_regions, regions, &mut output);
        out.push(output);
        out.extend(leaves);
    }

    fn node_properties(
        &self,
        id: types::NodeId,
        n: &crate::tree::node::HostNode,
    ) -> Option<Box<kivact_types::Semantics<kivact_types::SemanticsId>>> {
        let mut extras = n
            .semantics()
            .map(|props| Box::new(properties(props, |t| self.semantic_target(t))));
        let invalid = n.semantics().and_then(|s| s.invalid).unwrap_or(false);
        if invalid || n.read_only() || n.password() {
            let props = extras.get_or_insert_with(Default::default);
            props.invalid = Some(invalid);
            props.read_only = Some(props.read_only.unwrap_or(false) || n.read_only());
            props.protected = n.password();
        }
        if let Some(props) = &mut extras {
            self.validate_relations(
                types::FocusTarget {
                    node: id,
                    region: None,
                },
                props,
            );
        }
        extras
    }
    fn build_region_leaves(
        &mut self,
        id: types::NodeId,
        scene_regions: BTreeMap<types::RegionId, types::scene::Region>,
        regions: &mut BTreeSet<u64>,
        output: &mut kivact_types::render::SemanticsNode,
    ) -> Vec<kivact_types::render::SemanticsNode> {
        let hidden = output.semantics.hidden;
        let disabled = output.semantics.disabled;
        let bounds = output.bounds;
        let mut leaves = Vec::new();
        for (region_id, region) in scene_regions {
            let native_id = self.region_identity(id, region_id);
            regions.insert(native_id);
            output.children.push(kivact_types::SemanticsId(native_id));
            let offered = if hidden {
                Vec::new()
            } else {
                self.region_actions(id, region_id)
            };
            let region_focusable = !hidden
                && !region.semantics.hidden
                && !disabled
                && !region.semantics.disabled
                && region.semantics.focusable
                && self.in_scope(id);
            let mut props = Some(properties(&region.semantics, |t| self.semantic_target(t)));
            if region.semantics.invalid.unwrap_or(false) {
                props.get_or_insert_with(Default::default).invalid = Some(true);
            }
            if let Some(props) = &mut props {
                self.validate_relations(
                    types::FocusTarget {
                        node: id,
                        region: Some(region_id),
                    },
                    props,
                );
            }
            leaves.push(kivact_types::render::SemanticsNode {
                id: kivact_types::SemanticsId(native_id),
                children: Vec::new(),
                bounds: kivact_types::Rect {
                    x: bounds.x + region.rect.x,
                    y: bounds.y + region.rect.y,
                    width: region.rect.width,
                    height: region.rect.height,
                },
                scroll: kivact_types::Point::default(),
                transform: kivact_types::Transform::IDENTITY,
                semantics: kivact_types::Semantics {
                    role: region.semantics.role,
                    name: region.semantics.name,
                    description: Some(region.semantics.description.unwrap_or_default()),
                    value_text: region.semantics.value_text,
                    disabled: disabled || region.semantics.disabled,
                    hidden: hidden || region.semantics.hidden,
                    focusable: region_focusable,
                    checked: region.semantics.checked,
                    selected: region.semantics.selected,
                    expanded: region.semantics.expanded,
                    test_id: None,
                    actions: Some(offered),
                    ..(props).unwrap_or_default()
                },
            });
        }
        leaves
    }
    /// The transform the accessibility node of `id` carries (G-07: the one
    /// it is drawn and hit with), about its content-space box `bounds`
    /// (its layout box moved by the scrolls above it, `offset`). A layer
    /// (an overlay, a fixed box) is drawn at its window origin whatever
    /// transforms its ancestors have, so under a transformed ancestor it
    /// also undoes theirs (the platform maps a node by its ancestors'
    /// transforms).
    fn a11y_transform(
        &self,
        id: types::NodeId,
        layer: bool,
        bounds: types::Rect,
        offset: types::Point,
    ) -> kivact_types::Transform {
        let host = &self;
        let own = host.transform_at(id, bounds);
        if !layer {
            return own.unwrap_or(kivact_types::Transform::IDENTITY);
        }
        let ancestors: Vec<types::NodeId> =
            host.tree().chain(Some(id)).into_iter().skip(1).collect();
        let transformed = |x: &types::NodeId| host.node(*x).is_some_and(|n| n.place.transformed);
        if !ancestors.iter().any(transformed) {
            return own.unwrap_or(kivact_types::Transform::IDENTITY);
        }
        // What the platform maps it by: each ancestor's scroll, then
        // transform, nearest first.
        let mut around = kivact_types::Transform::IDENTITY;
        for x in &ancestors {
            let s = host.node(*x).map(|n| n.scroll()).unwrap_or_default();
            let above = self.scroll_above(*x);
            let r = host.rect(*x).unwrap_or_default();
            let content = types::Rect {
                x: r.x + above.x,
                y: r.y + above.y,
                ..r
            };
            around = around.then(kivact_types::Transform::translate(-s.x, -s.y));
            if let Some(t) = host.transform_at(*x, content) {
                around = around.then(t);
            }
        }
        // Where it is shown: its content box moved back by the scrolls
        // above it, then its own transform at its placed origin.
        let mut shown = kivact_types::Transform::translate(-offset.x, -offset.y);
        if let Some(r) = host.rect(id)
            && let Some(t) = host.transform_at(id, r)
        {
            shown = shown.then(t);
        }
        match around.invert() {
            Some(back) => shown.then(back),
            None => kivact_types::Transform::IDENTITY,
        }
    }

    /// Rows of every clipping list-like container that lie entirely
    /// outside it while neither neighbouring row reaches into it.
    pub(super) fn clipped_rows(&self) -> BTreeSet<types::NodeId> {
        let mut clipped = BTreeSet::new();
        for (list, _) in self.tree().iter() {
            if let Some((_, c)) = self.list_rows(list) {
                clipped.extend(c);
            }
        }
        clipped
    }

    /// The rows of `list` when it is a shown clipping list-like container,
    /// and those clipped out of it: lying entirely outside it while neither
    /// neighbouring row reaches into it. Containers without a role between
    /// the list and its rows are transparent; rows holding the focus stay
    /// exposed.
    fn list_rows(
        &self,
        list: types::NodeId,
    ) -> Option<(Vec<types::NodeId>, BTreeSet<types::NodeId>)> {
        let tree = self.tree();
        let n = tree.get(list)?;
        if !is_clipping_list(n, self.editors[n.index.0].is_some()) || !tree.shown(list) {
            return None;
        }
        let focus = tree.chain(self.focused().map(|f| f.node));
        let bounds = self.rect(list).unwrap_or_default();
        let mut rows = Vec::new();
        let mut stack: Vec<_> = tree
            .flat_children(tree.nodes.id_at(n.index))
            .rev()
            .collect();
        while let Some(id) = stack.pop() {
            let Some(child) = tree.get(id) else {
                continue;
            };
            if child.hidden() || child.visibility_hidden() {
                continue;
            }
            if semantic_role(child, self.editors[child.index.0].is_some()).is_none()
                && self.focused().map(|f| f.node) != Some(id)
            {
                stack.extend(tree.flat_children(tree.nodes.id_at(child.index)).rev());
            } else {
                rows.push(id);
            }
        }
        let inside: Vec<_> = rows
            .iter()
            .map(|row| overlaps(self.rect(*row).unwrap_or_default(), bounds))
            .collect();
        let mut clipped = BTreeSet::new();
        for (i, row) in rows.iter().enumerate() {
            let near = inside[i]
                || i.checked_sub(1).is_some_and(|before| inside[before])
                || inside.get(i + 1).copied().unwrap_or(false);
            if !near && !focus.contains(row) {
                clipped.insert(*row);
            }
        }
        Some((rows, clipped))
    }
}

/// A node's or a region's rare semantics as Arvane's, its relations'
/// targets resolved by `resolve` (one the platform tree cannot name is
/// left out; `validate_relations` in `build` drops the rest it must not
/// publish). Level and position count from 1, as in ARIA: `0` is "not
/// set" and is not published (for a client building the shared itself too).
/// A set size of `0` is published, an empty set. A position past the set's
/// size is not a valid pair: only the size is published, so assistive
/// technology never gets an invalid pair. The other values pass through as
/// they are.
pub(super) fn properties(
    p: &kivact_types::Semantics<kivact_types::FocusTarget>,
    resolve: impl Fn(types::FocusTarget) -> Option<kivact_types::SemanticsId>,
) -> kivact_types::Semantics<kivact_types::SemanticsId> {
    let mut mapped = p.clone().map_relations(resolve);
    mapped.level = mapped.level.filter(|&v| v != 0);
    mapped.pos_in_set = mapped
        .pos_in_set
        .filter(|&v| v != 0 && mapped.set_size.is_none_or(|size| v <= size));
    mapped.row_index = mapped.row_index.filter(|&value| value != 0);
    mapped.column_index = mapped.column_index.filter(|&value| value != 0);
    mapped.text = None;
    mapped
}

/// A clipping container with a list-like role, whose clipped rows a
/// virtual list mounts with overscan.
fn is_clipping_list(n: &crate::tree::node::HostNode, editor_present: bool) -> bool {
    n.clips() && semantic_role(n, editor_present).is_some_and(is_list_role)
}

/// A node that is not exposed (no role): only what the tree's shape needs,
/// its children (no scene regions), bounds, scroll, transform and
/// visibility.
pub(super) fn structural(
    id: types::NodeId,
    children: Vec<kivact_types::SemanticsId>,
    bounds: kivact_types::Rect,
    scroll: kivact_types::Point,
    transform: kivact_types::Transform,
    hidden: bool,
) -> kivact_types::render::SemanticsNode {
    kivact_types::render::SemanticsNode {
        id: kivact_types::SemanticsId(id.0),
        children,
        bounds,
        scroll,
        transform,
        semantics: kivact_types::Semantics {
            role: Some(kivact_types::Role::Generic),
            hidden,
            ..Default::default()
        },
    }
}

pub(super) fn drawable(r: types::Rect) -> bool {
    [r.x, r.y, r.width, r.height, r.x + r.width, r.y + r.height]
        .iter()
        .all(|v| v.is_finite())
        && r.width >= 0.0
        && r.height >= 0.0
}

/// Positive-area intersection, as AccessKit tests bounding boxes.
fn overlaps(a: types::Rect, b: types::Rect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

fn scene_regions(
    n: &crate::tree::node::HostNode,
) -> BTreeMap<types::RegionId, types::scene::Region> {
    n.scenes()
        .iter()
        .flatten()
        .flat_map(|scene| scene.regions.iter())
        // A client region with a non-finite or negative rect would
        // invalidate the whole tree; only that region is left out.
        .filter(|region| drawable(region.rect))
        .map(|region| (region.id, region.clone()))
        .collect()
}

fn focus_actions(
    hidden: bool,
    focusable: bool,
    offered: Vec<kivact_types::A11yAction>,
) -> Vec<kivact_types::A11yAction> {
    let mut actions: Vec<kivact_types::A11yAction> = if hidden { Vec::new() } else { offered };
    if focusable && !actions.contains(&kivact_types::A11yAction::Focus) {
        actions.push(kivact_types::A11yAction::Focus);
    }
    actions
}
