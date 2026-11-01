//! Role: Accessibility tree built from the model's semantics and handed to the window.
//!
//! The accessibility tree handed to the window: the model's semantics
//! policy (`model::semantics`: the role, the accessible name, the actions
//! a node and its scene regions offer) mapped onto Arvane's semantics,
//! which the window hands to AccessKit; and assistive technology's
//! requests mapped back onto the model.
//!
//! Invariants:
//! - What a node is exposed with is the model's
//!   (`Model::exposed`: role, name, actions): nothing offered while it
//!   is disabled (the node itself, never its descendants), not shown, or
//!   outside the shown modal; scene regions likewise. A tab stop in scope
//!   also offers `Focus`, which AccessKit needs to move the focus.
//! - A node without a role is not exposed: it stays in the tree only as a
//!   bare generic container for its children (AccessKit's platform
//!   adapters leave such a node out and lift its children to the parent),
//!   with no name, state, test id, actions, focus or scene regions, and
//!   assistive technology's requests on it are ignored.
//! - Bounds are untransformed layout boxes in content space (the scrolls
//!   above a node undone); a scroll container carries its offset and a
//!   transformed node the transform around its box, the one it is drawn
//!   and hit with (G-07), so the platform maps it as it is drawn (`build`).
//! - Sync records own-node, subtree, structural and focus dirt. Publication
//!   enumerates identity/order metadata, constructs only dirty payloads, and
//!   compares complete fingerprints. Only window acceptance commits hashes and
//!   clears dirt. No accepted payload tree is retained in the engine.
//!   A rejected update keeps its dirt and requests a complete resynchronization.
//! - Publication happens after the client's turns, or, when the native
//!   window has a frame to draw, after that frame
//!   (`Window::defers_accessibility`).
//! - Scene regions are flat synthetic leaves of their node, with native ids
//!   counted down from `u64::MAX` that stay stable while the region exists;
//!   a region with a non-finite or negative rect is left out alone.
//! - Rows of a clipping list-like container (virtual list overscan) lying
//!   wholly outside it beyond one neighbour are hidden, unless they hold
//!   the focus (AccessKit's `clips_children` filter).
//! - AccessKit has one default action: `Activate`, `Toggle` and `Select`
//!   resolve to the first one offered and activate the target.
use crate::model::Model;
use kivact_wire as wire;
mod build;
mod hash;
pub(crate) mod state;
mod text;

impl Model {
    /// Carry out assistive technology's request on a node or canvas region,
    /// if the node (or region) offers it now. A node that is not exposed
    /// (no role) takes no request, for itself or its regions.
    pub(crate) fn accessibility_action(
        &mut self,
        native: u64,
        action: kivact_types::A11yAction,
        value: Option<String>,
    ) {
        use wire::A11yAction as A;
        let (id, region) = self
            .a11y
            .native_regions
            .get(&native)
            .copied()
            .map_or((wire::NodeId(native), None), |(id, region)| {
                (id, Some(region))
            });
        let Some(exposed) = self.exposed(id) else {
            return;
        };
        let offered = match region {
            Some(region) => self.region_actions(id, region),
            None => exposed.actions,
        };
        match action {
            A::Focus => {
                // Fails (keeping the focus) off a tab stop or outside the modal.
                let _ = self.request_focus(wire::FocusTarget { node: id, region }, true);
            }
            action @ (A::Activate | A::Toggle | A::Select) => {
                // AccessKit exposes Activate/Toggle/Select as Click and sends
                // Activate back. Prefer an offered Activate for legacy controls;
                // otherwise recover the separately offered semantic action.
                let explicit = if action != A::Activate && offered.contains(&action) {
                    Some(action)
                } else if !offered.contains(&A::Activate) {
                    [A::Toggle, A::Select]
                        .into_iter()
                        .find(|a| offered.contains(a))
                } else {
                    None
                };
                if let Some(action) = explicit {
                    self.push(wire::HostEvent::A11y {
                        node: id,
                        region,
                        action,
                        value,
                    });
                } else if offered.contains(&A::Activate) {
                    self.push(wire::HostEvent::Activate {
                        node: id,
                        region,
                        pointer: None,
                    });
                }
            }
            A::SetValue if region.is_none() && offered.contains(&A::SetValue) => {
                // Explicit spin actions normalize numbers through their writer;
                // ordinary native editors still receive text like typing.
                let input = self.node(id).is_some_and(|n| {
                    self.editors[n.index.0]
                        .as_deref()
                        .map(|editor| editor.editor.state())
                        .is_some()
                        && !(n.role() == Some(wire::Role::SpinButton)
                            && n.actions().contains(&A::SetValue))
                });
                match value {
                    // A text input's value is the host's: set like typing.
                    Some(v) if input => self.set_input_value(id, &v),
                    value => self.push(wire::HostEvent::A11y {
                        node: id,
                        region,
                        action: A::SetValue,
                        value,
                    }),
                }
            }
            action if offered.contains(&action) => self.push(wire::HostEvent::A11y {
                node: id,
                region,
                action,
                value,
            }),
            _ => {}
        }
        self.sync();
    }
}

/// A node's or a region's rare semantics as Arvane's, its relations'
/// targets resolved by `resolve` (one the platform tree cannot name is
/// left out; `validate_relations` in `build` drops the rest it must not
/// publish). Level and position count from 1, as in ARIA: `0` is "not
/// set" and is not published (for a client building the wire itself too).
/// A set size of `0` is published, an empty set. A position past the set's
/// size is not a valid pair: only the size is published, so assistive
/// technology never gets an invalid pair. The other values pass through as
/// they are.
pub(crate) fn properties(
    p: &wire::SemanticsProperties,
    resolve: impl Fn(wire::FocusTarget) -> Option<kivact_types::NodeId>,
) -> kivact_types::SemanticsProperties {
    // Level and position are counted from 1, so 0 means "not set". A position
    // past the set's size is not a valid pair: the size is published, the
    // position is not.
    let level = p.level.filter(|&v| v != 0);
    let pos_in_set = p
        .pos_in_set
        .filter(|&v| v != 0 && p.set_size.is_none_or(|size| v <= size));
    kivact_types::SemanticsProperties {
        text: None,
        indeterminate: p.indeterminate,
        value_min: p.value_min,
        value_max: p.value_max,
        value_now: p.value_now,
        live_region: p.live_region,
        sort: p.sort,
        orientation: p.orientation,
        active_descendant: p.active_descendant.and_then(&resolve),
        described_by: p
            .described_by
            .iter()
            .filter_map(|id| resolve(*id))
            .collect(),
        controls: p.controls.iter().filter_map(|id| resolve(*id)).collect(),
        labelled_by: p.labelled_by.iter().filter_map(|id| resolve(*id)).collect(),
        level,
        set_size: p.set_size,
        pos_in_set,
        multiselectable: p.multiselectable,
        invalid: p.invalid,
        read_only: p.read_only,
        protected: p.protected,
    }
}

impl Model {
    pub(super) fn next_semantics_identity(&mut self) -> u64 {
        self.a11y.next_identity(&self.tree)
    }
}

impl state::A11yState {
    /// A new native identity: no node of `tree`, no scene region, and
    /// never given before.
    pub(super) fn next_identity(&mut self, tree: &crate::tree::NodeTree) -> u64 {
        loop {
            let candidate = self.next_semantics_id;
            self.next_semantics_id = candidate.checked_sub(1).expect(
                "invariant: native semantics identity space cannot exhaust in a process lifetime",
            );
            // The counter only decreases, so this candidate cannot be an earlier synthetic ID.
            if !tree.contains(wire::NodeId(candidate))
                && !self.native_regions.contains_key(&candidate)
            {
                return candidate;
            }
        }
    }
}

impl Model {
    /// Construct pending dirty payloads and submit them to the shell.
    ///
    /// Initial, reactivated, new-root and rejected publications send a complete
    /// replacement. Other updates send changed fingerprints in parent-before-child
    /// order; a parent's changed children detach removed descendants in the shell.
    /// Focus alone may send an empty delta. IDs can be allocated before acceptance,
    /// but hashes, clipping/dependency metadata and dirt commit only on acceptance.
    pub(crate) fn accessibility<W: arvane::Window>(
        &mut self,
        window: &mut W,
        active: bool,
        engine: &mut arvane::Engine,
    ) {
        if !active {
            self.a11y.root = None;
            self.a11y.invalidate();
            return;
        }
        if !self.a11y.pending && self.a11y.root.is_some() {
            return;
        }
        if self.tree().root().map(|id| kivact_types::NodeId(id.0)) != self.a11y.root {
            self.a11y.full = true;
        }
        let Some(mut candidate) = self.build_dirty_semantics(engine) else {
            self.a11y.invalidate();
            return;
        };
        let hashes: Vec<_> = candidate
            .nodes
            .iter()
            .map(|node| (node.id.0, hash::fingerprint(node)))
            .collect();
        let replace = self.a11y.full || self.a11y.root != Some(candidate.root);
        if !replace {
            candidate
                .nodes
                .retain(|node| self.a11y.hashes.get(&node.id.0) != Some(&hash::fingerprint(node)));
        }
        #[cfg(feature = "testing")]
        {
            self.a11y.published += candidate.nodes.len() as u64;
        }
        let (root, focus) = (candidate.root, candidate.focus);
        let accepted =
            if !replace && candidate.nodes.is_empty() && self.a11y.focus == candidate.focus {
                Ok(())
            } else if replace {
                window.replace_accessibility(&candidate)
            } else {
                window.update_accessibility(&kivact_types::SemanticsUpdate {
                    root,
                    focus,
                    nodes: candidate.nodes,
                })
            };
        if accepted.is_ok() {
            self.a11y.hashes.extend(hashes);
            if let Some(metadata) = self.a11y.prepared.take() {
                self.a11y.hashes.retain(|id, _| metadata.live.contains(id));
                self.a11y
                    .region_ids
                    .retain(|key, _| metadata.regions.contains(key));
                self.a11y
                    .native_regions
                    .retain(|_, key| metadata.regions.contains(key));
                self.a11y
                    .authored
                    .retain(|key, _| metadata.authored.contains(key));
                self.a11y.metadata = std::rc::Rc::new(metadata);
            }
            self.a11y.metadata_refresh = false;
            self.a11y.root = Some(root);
            self.a11y.focus = focus;
            self.a11y.pending = false;
            self.a11y.full = false;
            self.a11y.nodes.clear();
            self.a11y.subtrees.clear();
            self.a11y.removed.clear();
        } else {
            self.a11y.invalidate();
        }
    }
}
