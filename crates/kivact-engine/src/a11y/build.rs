//! Role: Construct dirty accessibility payloads using the full builder's policy.
//!
//! Identity/parent order, clipping and raw relation dependencies are enumerated
//! without constructing payloads. Subtree dirt and reverse edges expand once;
//! only selected nodes call build_node. The independent debug reference calls
//! the same policy for every node with disposable metadata and EditLines.
//!
//! Invariants:
//! - Bounds are the untransformed content-space box: the node's layout box
//!   with the scrolls of its ancestors undone. A scroll container carries
//!   its offset, a transformed node the transform about its box. A layer
//!   (an overlay, a fixed box) is drawn at its window origin whatever its
//!   ancestors' transforms, so under a transformed ancestor it carries the
//!   transform that undoes theirs.
//! - A relation (`described_by`, `controls`, `labelled_by`) keeps, in the
//!   order kivact wrote them, only the targets the published tree has: a
//!   node (or a scene region's node) that exists, is attached to the root
//!   and is exposed (has a role). A hidden target stays, as ARIA allows.
//!   Old and current raw reverse edges invalidate sources when targets change,
//!   including relations temporarily filtered from the accepted payload.
//! - An active descendant is kept only while it is shown, has a role (a
//!   region: neither it nor its node disabled, and the region not hidden)
//!   and lies in the node's subtree or in the subtree of a node it
//!   `controls` (the node itself included: the ARIA combobox pattern of a
//!   text field and its sibling list). The controls are checked first, so
//!   a dropped relation does not admit one.
//! - Level, set size and position follow `properties` (one-based, `0`
//!   unset, no position past the set's size).
//! - A text input publishes no value (that would copy and compare its whole
//!   text): its text goes as the text runs of the lines around its caret
//!   and those shown (`editing_semantics`). A password publishes neither its
//!   value nor a text snapshot. Assistive technology can set an input's
//!   value (`SetValue`, as typing), but a request to select a range is not
//!   supported: no such action exists.

use std::collections::{BTreeMap, BTreeSet};

use arvane as a;

use crate::model::Model;
use crate::{is_list_role, semantic_role};
use kivact_wire as wire;

mod dirty;

impl Model {
    /// The focus the tree reports: the focused node when it is exposed,
    /// shown and focusable.
    fn semantics_focus(&self, clipped: &BTreeSet<wire::NodeId>) -> Option<kivact_types::NodeId> {
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
                .map(kivact_types::NodeId),
            None => Some(kivact_types::NodeId(f.node.0)),
        }
    }

    /// Attached to the root (an overlay through its parent).
    fn attached(&self, id: wire::NodeId) -> bool {
        self.tree().chain(Some(id)).last().copied() == self.tree().root()
    }

    /// The scroll offsets of the ancestors of `id`, summed.
    fn scroll_above(&self, id: wire::NodeId) -> wire::Point {
        let mut out = wire::Point::default();
        // From the root down, summed in the order a whole build sums them.
        for x in self.tree().chain(Some(id)).into_iter().skip(1).rev() {
            let s = self.node(x).map(|n| n.scroll()).unwrap_or_default();
            out.x += s.x;
            out.y += s.y;
        }
        out
    }

    fn hidden(&self, id: wire::NodeId, clipped: &BTreeSet<wire::NodeId>) -> bool {
        !self.tree().shown(id) || clipped.contains(&id)
    }

    fn focusable(&self, id: wire::NodeId, hidden: bool) -> bool {
        !hidden && self.is_tab_stop(id) && self.in_scope(id)
    }

    /// Build node `id` (and its scene regions' leaves) into `out`;
    /// `offset` is the scroll above it. The native ids of its regions go
    /// into `regions`.
    fn build_node(
        &mut self,
        id: wire::NodeId,
        offset: wire::Point,
        clipped: &BTreeSet<wire::NodeId>,
        out: &mut Vec<kivact_types::SemanticsNode>,
        regions: &mut BTreeSet<u64>,
        engine: &mut a::Engine,
    ) {
        let Some(n) = self.node(id) else {
            return;
        };
        let hidden = self.hidden(id, clipped);
        let rect = self.rect(id).unwrap_or_default();
        let bounds = wire::Rect {
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
        let children: Vec<kivact_types::NodeId> = self
            .tree()
            .children_ids(n)
            .map(|id| kivact_types::NodeId(id.0))
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
        let mut actions: Vec<kivact_types::A11yAction> =
            if hidden { Vec::new() } else { exposed.actions };
        if focusable && !actions.contains(&kivact_types::A11yAction::Focus) {
            actions.push(kivact_types::A11yAction::Focus);
        }
        let scene_regions: BTreeMap<_, _> = n
            .scenes()
            .iter()
            .flatten()
            .flat_map(|scene| scene.regions.iter())
            // A client region with a non-finite or negative rect would
            // invalidate the whole tree; only that region is left out.
            .filter(|region| drawable(region.rect))
            .map(|region| (region.id, region.clone()))
            .collect();
        let mut extras = n
            .semantics()
            .and_then(|s| s.accessibility.as_deref())
            .map(|props| Box::new(super::properties(props, |t| self.semantic_target(t))));
        let invalid = n.semantics().and_then(|s| s.invalid).unwrap_or(false);
        if invalid || n.read_only() || n.password() {
            let props = extras.get_or_insert_with(Default::default);
            props.invalid = invalid;
            props.read_only = n.read_only();
            props.protected = n.password();
        }
        if let Some(props) = &mut extras {
            self.validate_relations(
                wire::FocusTarget {
                    node: id,
                    region: None,
                },
                props,
            );
        }
        let authored_text = n
            .semantics()
            .and_then(|s| s.accessibility.as_deref())
            .and_then(|props| props.text.clone());
        let mut output = kivact_types::SemanticsNode {
            properties: extras,
            id: kivact_types::NodeId(id.0),
            role: exposed.role,
            children,
            label: exposed.name,
            description: n
                .semantics()
                .and_then(|s| s.description.clone())
                .unwrap_or_default(),
            // An editor's text is published as its text runs (around its
            // caret and what it shows: `editing_semantics`), not copied
            // and compared as a value.
            value: match self.editors[n.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())
            {
                Some(_) => None,
                None => n.semantics().and_then(|s| s.value.clone()),
            },
            bounds,
            scroll,
            transform,
            disabled,
            hidden,
            focusable,
            checked: n.semantics().and_then(|s| s.checked).flatten(),
            selected: n.selected(),
            expanded: n.semantics().and_then(|s| s.expanded).flatten(),
            test_id: n.semantics().and_then(|s| s.test_id.clone()),
            actions,
        };
        if self.node(id).is_some_and(|n| {
            self.editors[n.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())
                .is_some()
                && !n.password()
        }) {
            let text = self.editing_semantics(id, offset, engine);
            output.properties.get_or_insert_with(Default::default).text = Some(text);
        } else if let Some(authored) = authored_text {
            // A node that draws and edits its own text publishes its own runs.
            let text = self.authored_semantics(id, &authored, bounds);
            output.properties.get_or_insert_with(Default::default).text = text;
        }
        let mut leaves = Vec::new();
        for (region_id, region) in scene_regions {
            let state = region.properties.as_deref();
            let checked = state.and_then(|s| s.checked);
            let selected = state.is_some_and(|s| s.selected);
            let expanded = state.and_then(|s| s.expanded);
            let native_id = self.region_identity(id, region_id);
            regions.insert(native_id);
            output.children.push(kivact_types::NodeId(native_id));
            let offered = if hidden {
                Vec::new()
            } else {
                self.region_actions(id, region_id)
            };
            let region_focusable = !hidden
                && !region.hidden
                && !disabled
                && !region.disabled
                && region.focusable
                && self.in_scope(id);
            leaves.push(kivact_types::SemanticsNode {
                properties: region
                    .properties
                    .as_deref()
                    .map(|props| {
                        let mut props = super::properties(props, |t| self.semantic_target(t));
                        self.validate_relations(
                            wire::FocusTarget {
                                node: id,
                                region: Some(region_id),
                            },
                            &mut props,
                        );
                        props
                    })
                    .map(Box::new),
                id: kivact_types::NodeId(native_id),
                role: region.role,
                children: Vec::new(),
                label: region.name,
                description: region.description.unwrap_or_default(),
                value: region.value,
                bounds: kivact_types::Rect {
                    x: bounds.x + region.rect.x,
                    y: bounds.y + region.rect.y,
                    width: region.rect.width,
                    height: region.rect.height,
                },
                scroll: kivact_types::Point::default(),
                transform: kivact_types::Transform::IDENTITY,
                disabled: disabled || region.disabled,
                hidden: hidden || region.hidden,
                focusable: region_focusable,
                checked,
                selected,
                expanded,
                test_id: None,
                actions: offered,
            });
        }
        out.push(output);
        out.extend(leaves);
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
        id: wire::NodeId,
        layer: bool,
        bounds: wire::Rect,
        offset: wire::Point,
    ) -> kivact_types::Transform {
        let host = &self;
        let own = host.transform_at(id, bounds);
        if !layer {
            return own.unwrap_or(kivact_types::Transform::IDENTITY);
        }
        let ancestors: Vec<wire::NodeId> =
            host.tree().chain(Some(id)).into_iter().skip(1).collect();
        let transformed = |x: &wire::NodeId| host.node(*x).is_some_and(|n| n.place.transformed);
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
            let content = wire::Rect {
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
    fn clipped_rows(&self) -> BTreeSet<wire::NodeId> {
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
    fn list_rows(&self, list: wire::NodeId) -> Option<(Vec<wire::NodeId>, BTreeSet<wire::NodeId>)> {
        let tree = self.tree();
        let n = tree.get(list)?;
        if !is_clipping_list(n, self.editors[n.index.0].is_some()) || !tree.shown(list) {
            return None;
        }
        let focus = tree.chain(self.focused().map(|f| f.node));
        let bounds = self.rect(list).unwrap_or_default();
        let mut rows = Vec::new();
        let mut stack: Vec<_> = tree.children_ids(n).rev().collect();
        while let Some(id) = stack.pop() {
            let Some(child) = tree.get(id) else {
                continue;
            };
            if child.hidden() {
                continue;
            }
            if semantic_role(child, self.editors[child.index.0].is_some()).is_none()
                && self.focused().map(|f| f.node) != Some(id)
            {
                stack.extend(tree.children_ids(child).rev());
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

    /// Validate relationships after resolving node and region platform identities.
    fn validate_relations(
        &self,
        source: wire::FocusTarget,
        props: &mut kivact_types::SemanticsProperties,
    ) {
        // A relation names nodes the platform tree has: one that is
        // gone, not attached or not exposed (no role) is left out. A
        // hidden target stays, as `aria-describedby` allows.
        for ids in [
            &mut props.described_by,
            &mut props.controls,
            &mut props.labelled_by,
        ] {
            ids.retain(|target| {
                let target = self.semantic_owner(*target);
                target.is_some_and(|target| self.attached(target) && self.exposed(target).is_some())
            });
        }
        // The active descendant is shown, has a role and lies inside the
        // node or inside a node it controls (the ARIA combobox pattern: a
        // text field whose list is a sibling). The controls were resolved
        // just above, so a target of a dropped relation does not count.
        props.active_descendant = props.active_descendant.filter(|native| {
            let Some(target) = self.semantic_identity(*native) else {
                return false;
            };
            let Some(node) = self.node(target.node) else {
                return false;
            };
            if !self.tree().shown(target.node) || node.role().is_none() {
                return false;
            }
            if let Some(region) = target.region
                && (node.disabled() || node.region(region).is_none_or(|r| r.hidden || r.disabled))
            {
                return false;
            }
            let inside = |ancestor: wire::FocusTarget| match ancestor.region {
                Some(_) => target == ancestor,
                None => self.tree().is_in_subtree(target.node, ancestor.node),
            };
            inside(source)
                || props
                    .controls
                    .iter()
                    .filter_map(|c| self.semantic_identity(*c))
                    .any(inside)
        });
    }

    /// Resolve existing focus identities into the platform accessibility tree.
    fn semantic_target(&self, target: wire::FocusTarget) -> Option<kivact_types::NodeId> {
        self.node(target.node)?;
        match target.region {
            None => Some(kivact_types::NodeId(target.node.0)),
            Some(region) => {
                self.node(target.node)?.region(region)?;
                self.a11y
                    .region_ids
                    .get(&(target.node, region))
                    .copied()
                    .map(kivact_types::NodeId)
            }
        }
    }

    fn semantic_identity(&self, target: kivact_types::NodeId) -> Option<wire::FocusTarget> {
        if let Some((node, region)) = self.a11y.native_regions.get(&target.0) {
            self.node(*node)?.region(*region)?;
            Some(wire::FocusTarget {
                node: *node,
                region: Some(*region),
            })
        } else {
            self.node(wire::NodeId(target.0))?;
            Some(wire::FocusTarget {
                node: wire::NodeId(target.0),
                region: None,
            })
        }
    }

    fn semantic_owner(&self, target: kivact_types::NodeId) -> Option<wire::NodeId> {
        if let Some((node, region)) = self.a11y.native_regions.get(&target.0) {
            self.node(*node)?.region(*region)?;
            Some(*node)
        } else {
            self.node(wire::NodeId(target.0))
                .map(|_| wire::NodeId(target.0))
        }
    }

    fn prepare_region_ids(&mut self) {
        let targets: Vec<_> = self
            .tree()
            .preorder()
            .into_iter()
            .flat_map(|node| {
                self.node(node).into_iter().flat_map(move |n| {
                    n.scenes()
                        .iter()
                        .flatten()
                        .flat_map(move |s| s.regions.iter().map(move |r| (node, r.id)))
                })
            })
            .collect();
        for (node, region) in targets {
            self.region_identity(node, region);
        }
    }

    fn region_identity(&mut self, owner: wire::NodeId, region: wire::RegionId) -> u64 {
        if let Some(&id) = self.a11y.region_ids.get(&(owner, region))
            && !self.tree().contains(wire::NodeId(id))
        {
            return id;
        }
        let candidate = self.next_semantics_identity();
        self.a11y.native_regions.insert(candidate, (owner, region));
        self.a11y.region_ids.insert((owner, region), candidate);
        candidate
    }
}

/// A clipping container with a list-like role, whose clipped rows a
/// virtual list mounts with overscan.
fn is_clipping_list(n: &crate::tree::node::HostNode, editor_present: bool) -> bool {
    n.clips() && semantic_role(n, editor_present).is_some_and(is_list_role)
}

/// A node that is not exposed (no role): only what the tree's shape needs,
/// its children (no scene regions), bounds, scroll, transform and
/// visibility.
fn structural(
    id: wire::NodeId,
    children: Vec<kivact_types::NodeId>,
    bounds: kivact_types::Rect,
    scroll: kivact_types::Point,
    transform: kivact_types::Transform,
    hidden: bool,
) -> kivact_types::SemanticsNode {
    kivact_types::SemanticsNode {
        properties: None,
        id: kivact_types::NodeId(id.0),
        role: kivact_types::Role::Generic,
        children,
        label: String::new(),
        description: String::new(),
        value: None,
        bounds,
        scroll,
        transform,
        disabled: false,
        hidden,
        focusable: false,
        checked: None,
        selected: false,
        expanded: None,
        test_id: None,
        actions: Vec::new(),
    }
}

fn drawable(r: wire::Rect) -> bool {
    [r.x, r.y, r.width, r.height, r.x + r.width, r.y + r.height]
        .iter()
        .all(|v| v.is_finite())
        && r.width >= 0.0
        && r.height >= 0.0
}

/// Positive-area intersection, as AccessKit tests bounding boxes.
fn overlaps(a: wire::Rect, b: wire::Rect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}
