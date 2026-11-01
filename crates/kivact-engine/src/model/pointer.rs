//! Role: Pointer state machine: hover, press, capture, click, drag and wheel routing.
//!
//! The pointer state machine: one track per pointer (the mouse, each touch
//! or pen contact) with its hover and pressed chains, `Enter`/`Leave` of the
//! nearest pointer listener and pointer capture; press → focus, click →
//! `Activate`, drag selection in a text input, and wheel routing. The
//! caller hit-tests ([`Model::hit`]) and gives the pointer's window
//! position ([`PointerSpace`]).
//!
//! Invariants:
//! - A hit outside the scope of a shown modal hits nothing (the backdrop).
//! - Every pointer has a track of its own (`PointerState::tracks`): the
//!   mouse always, a touch or pen contact from its `Down` to its `Up` or
//!   `Cancel`. Its capture, `Enter`/`Leave` listener, hover and pressed
//!   chains and press target are its own; its events carry its id, kind,
//!   pressure and whether it is primary; no pointer waits for another.
//! - The primary pointers (Pointer Events' `isPrimary`: the mouse; the
//!   first touch and the first pen contact while no other of its kind is in
//!   contact) also make what is synthesized from the pointer: the hover and
//!   pressed state, `HoverChanged`, the cursor (the mouse's), and a press's
//!   outside dismissal, focus, caret and activation. Another contact only
//!   gets its pointer events, at the listener it captured.
//! - A pointer's hover chain is the hit node and its ancestors; its pressed
//!   chain is its hover chain at a primary press, until the release. A node
//!   is hovered while a primary pointer hovers it, and shows its pressed
//!   state only while a primary pointer that pressed it still hovers it, so
//!   a hover change repaints the pressed nodes it flips, hover style or not.
//! - `Enter` and `Leave` go to the nearest pointer listener under each
//!   pointer when it changes (moving between its children sends neither);
//!   `Move` goes to the pointer's capture, else to that listener once
//!   entered.
//! - `HoverChanged` reports the head of the first non-empty hover chain
//!   (the mouse's first, then the contacts' in the order they began: the
//!   hit itself, whatever listener or capture there is) whenever it
//!   changes. A commit clears a hit that got hidden or left the shown
//!   modal's scope, the removal of the hit clears it (its surviving
//!   ancestors are no new hit), and so does the mouse leaving the window
//!   with a button held. Otherwise only the pointer's own input hits again:
//!   a layout change under a pointer that stays still changes no hover.
//! - A primary pointer's press first dismisses the overlays it lands
//!   outside of, then, with no capture, any button first resolves the hit
//!   region or nearest declared pointer focus destination once, then falls
//!   back to its native focus target. Without a declaration, any button
//!   focuses the nearest focusable ancestor. A primary press on nothing
//!   focusable blurs (committing a composition), except on a modal's
//!   backdrop or inside a shown overlay whose anchor holds the focus
//!   ([`Model::press_blurs`]); a press of another button on nothing
//!   focusable, and any press while a capture is held, keep the focus. Only
//!   the primary button presses and activates.
//! - After its first `Down` the listener holds that pointer's capture: its
//!   later button `Down`, `Move`, each `Up` and `Cancel` go to it until its
//!   last held button releases, wherever the pointer is. A mouse move with
//!   no button held while a capture is active ends it with `Cancel`.
//! - A press on an enabled region offering `Activate` retains that exact region;
//!   `Activate` requires a release on the same still-available region. A
//!   disabled region blocks activation. Other presses retain the nearest
//!   activatable ancestor and activate its own action on a subtree release.
//! - Canceling one pointer ends only its own capture and press; losing the
//!   window's focus cancels them all (`Model::cancel_pointer`).

use std::cell::RefCell;

use kivact_wire::{
    A11yAction, FocusTarget, HostEvent, Modifiers, MouseButton, NodeId, Point, PointerButtons,
    PointerEvent, PointerId, PointerPhase, RegionId,
};

use crate::model::Model;

mod shell;
pub(crate) mod state;
mod touch;

use state::{CursorKind, PointerSource, Track};

/// Where the pointer of one input event is, relative to the nodes: a
/// window position, and the engine that finds the caret under it.
/// One concrete type, as `measure::leaf::Measure` is: window positions,
/// scene regions and shaped text offsets need no trait.
pub(crate) struct PointerSpace<'a> {
    at: Point,
    engine: RefCell<&'a mut arvane::Engine>,
}

impl<'a> PointerSpace<'a> {
    /// The pointer at the window position `at`.
    pub(crate) fn new(at: Point, engine: &'a mut arvane::Engine) -> Self {
        Self {
            at,
            engine: RefCell::new(engine),
        }
    }

    /// The pointer position in `node`'s local coordinates: relative to its
    /// untransformed border box ([`Model::rect`]), mapped back through
    /// its transforms (`Model::to_local`).
    pub(crate) fn local(&self, model: &Model, node: NodeId) -> Point {
        model.to_local(node, self.at)
    }

    /// The pointer position in the window.
    pub(crate) fn window(&self) -> Point {
        self.at
    }

    /// The scene region of `node` under the pointer: the topmost region of
    /// its scenes containing `local`.
    pub(crate) fn region(&self, model: &Model, node: NodeId, local: Point) -> Option<RegionId> {
        model.node(node)?.region_at(local)
    }

    /// The byte offset in the value of the text input `node` under the
    /// pointer (Arvane's shaping, `text_geometry`; `None`: the caret
    /// stays).
    pub(crate) fn text_offset(&self, model: &Model, node: NodeId) -> Option<usize> {
        let mut engine = self.engine.borrow_mut();
        crate::model::interact::text_geometry::offset_at(model, node, self.at, &mut engine)
    }
}

impl Model {
    /// Whether `id` is under a primary pointer (in its hover chain).
    pub(crate) fn is_hovered(&self, id: NodeId) -> bool {
        self.pointer
            .state
            .tracks
            .values()
            .any(|t| t.hover.contains(&id))
    }

    /// The nearest ancestor of `id` that is a group.
    pub(crate) fn nearest_group(&self, id: NodeId) -> Option<NodeId> {
        self.tree.chain(Some(id)).into_iter().skip(1).find(|x| {
            self.tree
                .get(*x)
                .and_then(|n| n.states())
                .is_some_and(|s| s.group)
        })
    }

    /// Whether the nearest ancestor of `id` that is a group is hovered.
    pub(crate) fn is_group_hovered(&self, id: NodeId) -> bool {
        self.nearest_group(id).is_some_and(|g| self.is_hovered(g))
    }

    /// The nodes below the groups among `nodes` whose nearest group is one
    /// of them and that have a group style `has` (a nested group itself,
    /// not below it).
    pub(crate) fn group_dependents(
        &self,
        nodes: &[NodeId],
        has: impl Fn(&kivact_wire::StateStyles) -> bool,
    ) -> Vec<NodeId> {
        nodes
            .iter()
            .filter(|g| self.is_group(**g))
            .flat_map(|g| self.group_members(*g, &has))
            .collect()
    }

    /// The nodes below `node` that have a group style `has` and no group
    /// between them and `node` (a nested group itself, not below it):
    /// what takes its group states from `node` while it is a group.
    pub(crate) fn group_members(
        &self,
        node: NodeId,
        has: impl Fn(&kivact_wire::StateStyles) -> bool,
    ) -> Vec<NodeId> {
        let mut out = Vec::new();
        let Some(n) = self.tree.get(node) else {
            return out;
        };
        let mut stack: Vec<NodeId> = self.tree.children_ids(n).collect();
        while let Some(x) = stack.pop() {
            let Some(c) = self.tree.get(x) else { continue };
            if c.states().is_some_and(&has) {
                out.push(x);
            }
            if !self.is_group(x) {
                stack.extend(self.tree.children_ids(c));
            }
        }
        out
    }

    /// Whether `id` is a group (`Style::group`).
    fn is_group(&self, id: NodeId) -> bool {
        self.tree
            .get(id)
            .and_then(|n| n.states())
            .is_some_and(|s| s.group)
    }

    /// Whether `id` shows its pressed state (pressed by a primary pointer
    /// that still hovers it).
    pub(crate) fn is_pressed(&self, id: NodeId) -> bool {
        self.pointer
            .state
            .tracks
            .values()
            .any(|t| t.pressed.contains(&id) && t.hover.contains(&id))
    }

    /// The node under the pointer: the head of the first non-empty hover
    /// chain (the mouse's first).
    pub(crate) fn hovered(&self) -> Option<NodeId> {
        self.pointer
            .state
            .tracks
            .values()
            .find_map(|t| t.hover.first().copied())
    }

    /// The cursor the mouse shows, decided by the first of the mouse's
    /// capture and then its hover chain (nearest the pointer first) that
    /// has one: the cursor its paint names (state styles and transitions
    /// included), else the text cursor for an enabled text input, the
    /// pointing hand for an activatable node; with none, the arrow.
    pub(crate) fn cursor(&self) -> CursorKind {
        let mouse = self.pointer.state.track(PointerId::MOUSE);
        for id in mouse.capture.into_iter().chain(mouse.hover.iter().copied()) {
            if let Some(n) = self.tree.get(id) {
                let specified = self.effective_paint(id).cursor;
                if specified != CursorKind::Auto {
                    return specified;
                }
                if self.editors[n.index.0]
                    .as_deref()
                    .map(|editor| editor.editor.state())
                    .is_some()
                    && !n.disabled()
                {
                    return CursorKind::Text;
                }
                if n.activatable() {
                    return CursorKind::Pointer;
                }
            }
        }
        CursorKind::Default
    }

    fn scoped(&self, hit: Option<NodeId>) -> Option<NodeId> {
        hit.filter(|h| self.tree.contains(*h) && self.in_scope(*h))
    }

    #[expect(clippy::too_many_arguments, reason = "one pointer event's fields")]
    fn send_pointer(
        &mut self,
        pointer: PointerId,
        node: NodeId,
        phase: PointerPhase,
        button: Option<MouseButton>,
        modifiers: Modifiers,
        click_count: u8,
        space: &PointerSpace<'_>,
    ) {
        let pos = space.local(self, node);
        let region = space.region(self, node, pos);
        let source = self.pointer.state.track(pointer).source;
        self.push(HostEvent::Pointer {
            node,
            region,
            event: PointerEvent {
                id: source.id,
                phase,
                pos,
                window_pos: space.window(),
                button: if matches!(phase, PointerPhase::Down | PointerPhase::Up) {
                    button
                } else {
                    None
                },
                buttons: if phase == PointerPhase::Cancel {
                    PointerButtons::NONE
                } else {
                    source.buttons
                },
                modifiers,
                click_count,
                kind: source.kind,
                pressure: source.pressure,
                tilt: source.tilt,
                is_primary: source.primary,
            },
        });
    }

    /// Follow a new contact, `source`, at the window position `at`.
    fn begin_pointer(&mut self, source: PointerSource, at: Point) {
        let mut track = Track::new(source);
        track.at = at;
        self.pointer.state.tracks.insert(source.id.0, track);
    }

    /// Stop following the contact `pointer` (it ended).
    fn end_pointer(&mut self, pointer: PointerId) {
        self.pointer.state.tracks.remove(&pointer.0);
    }

    /// Swap `pointer`'s pressed or hover chain with `chain`.
    fn swap_chain(&mut self, pointer: PointerId, pressed: bool, chain: &mut Vec<NodeId>) {
        let track = self.pointer.state.track_mut(pointer);
        let own = if pressed {
            &mut track.pressed
        } else {
            &mut track.hover
        };
        std::mem::swap(own, chain);
    }

    /// Replace `pointer`'s pressed or hover chain, repainting the nodes
    /// whose shown hover or pressed state flips.
    fn set_chain(&mut self, pointer: PointerId, pressed: bool, mut chain: Vec<NodeId>) {
        let track = self.pointer.state.track(pointer);
        let old = if pressed {
            &track.pressed
        } else {
            &track.hover
        };
        if *old == chain {
            return;
        }
        let touched: Vec<NodeId> = old
            .iter()
            .filter(|x| !chain.contains(x))
            .chain(chain.iter().filter(|x| !old.contains(x)))
            .copied()
            .collect();
        // The hover of a group restyles the nodes below it.
        let dependents = if pressed {
            Vec::new()
        } else {
            self.group_dependents(&touched, |s| s.group_hover.is_some())
        };
        let shown = |model: &Model, id: NodeId| (model.is_hovered(id), model.is_pressed(id));
        let before: Vec<_> = touched.iter().map(|id| shown(self, *id)).collect();
        let head = self.hovered();
        self.swap_chain(pointer, pressed, &mut chain);
        // (node, whether its hover flipped, whether its shown pressed state flipped)
        let flipped: Vec<(NodeId, bool, bool)> = touched
            .iter()
            .zip(before)
            .filter_map(|(id, (hovered, shows_pressed))| {
                let (now_hovered, now_pressed) = shown(self, *id);
                let (hover, press) = (now_hovered != hovered, now_pressed != shows_pressed);
                (hover || press).then_some((*id, hover, press))
            })
            .collect();
        if !flipped.is_empty() || !dependents.is_empty() {
            // The motion starts from what is shown before the change.
            self.swap_chain(pointer, pressed, &mut chain);
            for id in flipped
                .iter()
                .map(|(id, ..)| *id)
                .chain(dependents.iter().copied())
            {
                self.prepare_state_motion(id);
            }
            self.swap_chain(pointer, pressed, &mut chain);
        }
        for id in dependents {
            self.restyled(id);
        }
        for (id, hover, press) in flipped {
            let has =
                self.tree.get(id).and_then(|n| n.states()).is_some_and(|s| {
                    (hover && s.hover.is_some()) || (press && s.pressed.is_some())
                });
            if has {
                self.repaint(id);
                if self
                    .tree
                    .get(id)
                    .and_then(|n| n.states())
                    .is_some_and(crate::tree::states_place)
                {
                    // A state transform or z-index moves or reorders it:
                    // reported and placed again like a written one.
                    self.changes.needs_layout = true;
                    self.changes.moved = true;
                }
            }
        }
        let now = self.hovered();
        if now != head {
            self.push(HostEvent::HoverChanged { node: now });
        }
    }

    /// `pointer` hovers nothing, keeping its capture and the listener it
    /// entered (the mouse left the window with a button held, the hit was
    /// removed).
    pub(crate) fn clear_hover(&mut self, pointer: PointerId) {
        self.set_chain(pointer, false, Vec::new());
    }

    /// At a commit: a hit that got hidden (by itself or an ancestor) or
    /// lies outside the shown modal is hovered no more.
    pub(crate) fn revalidate_hover(&mut self) {
        self.revalidate_touch_scrolls();
        self.clear_hits_where(|model, hit| !model.tree.shown(hit) || !model.in_scope(hit));
    }

    /// A removed hit is hovered no more (its surviving ancestors are no new
    /// hit).
    pub(crate) fn clear_removed_hits(&mut self) {
        self.clear_hits_where(|model, hit| !model.tree.contains(hit));
    }

    /// Clear the hover of every pointer whose hit is `stale`.
    fn clear_hits_where(&mut self, stale: impl Fn(&Model, NodeId) -> bool) {
        let pointers: Vec<PointerId> = self
            .pointer
            .state
            .tracks
            .values()
            .filter(|t| t.hover.first().is_some_and(|hit| stale(self, *hit)))
            .map(|t| t.source.id)
            .collect();
        for pointer in pointers {
            self.clear_hover(pointer);
        }
    }

    /// `pointer`'s hover chain (a primary pointer's) and `Enter`/`Leave` of
    /// the listener under `hit`.
    fn track_hover(
        &mut self,
        pointer: PointerId,
        hit: Option<NodeId>,
        modifiers: Modifiers,
        space: &PointerSpace<'_>,
    ) {
        if self.pointer.state.track(pointer).source.primary {
            let chain = self.tree.chain(hit);
            self.set_chain(pointer, false, chain);
        }
        let over = hit.and_then(|h| self.ancestor_where(h, |n| n.listens_pointer()));
        let old = std::mem::replace(&mut self.pointer.state.track_mut(pointer).over, over);
        if over != old {
            if let Some(o) = old.filter(|o| self.tree.contains(*o)) {
                self.send_pointer(pointer, o, PointerPhase::Leave, None, modifiers, 0, space);
            }
            if let Some(n) = over {
                self.send_pointer(pointer, n, PointerPhase::Enter, None, modifiers, 0, space);
            }
        }
    }

    /// Resolve one declaration without following the destination's own policy.
    fn pointer_focus_destination(
        &self,
        hit: NodeId,
        space: &PointerSpace<'_>,
    ) -> Option<FocusTarget> {
        let region = space.region(self, hit, space.local(self, hit));
        let declared = region
            .and_then(|id| self.node(hit)?.region(id)?.focus_on_press)
            .or_else(|| {
                self.ancestor_where(hit, |n| n.focus_on_press().is_some())
                    .and_then(|id| self.node(id)?.focus_on_press())
            });
        declared.filter(|target| self.is_focus_target(*target) && self.in_scope(target.node))
    }

    /// A button of `pointer` pressed over `hit` (the host's hit test).
    pub(crate) fn pointer_down(
        &mut self,
        pointer: PointerId,
        hit: Option<NodeId>,
        button: MouseButton,
        modifiers: Modifiers,
        click_count: u8,
        space: &PointerSpace<'_>,
    ) {
        let hit = self.scoped(hit);
        self.track_hover(pointer, hit, modifiers, space);
        let capture = self.pointer.state.track(pointer).capture;
        if self.pointer.state.track(pointer).source.primary {
            self.primary_press(pointer, hit, button, modifiers, capture, space);
        }
        let target =
            capture.or_else(|| hit.and_then(|h| self.ancestor_where(h, |n| n.listens_pointer())));
        if let Some(t) = target.filter(|t| self.tree.get(*t).is_some_and(|n| !n.disabled())) {
            self.pointer.state.track_mut(pointer).capture = Some(t);
            self.send_pointer(
                pointer,
                t,
                PointerPhase::Down,
                Some(button),
                modifiers,
                click_count,
                space,
            );
        }
    }

    /// What a primary pointer's press over `hit` does besides its `Down`:
    /// dismiss the overlays it lands outside of, focus (or blur), retain
    /// the pressed chain and the activation target, and place a text
    /// input's caret. `capture` is the pointer's capture before the press.
    fn primary_press(
        &mut self,
        pointer: PointerId,
        hit: Option<NodeId>,
        button: MouseButton,
        modifiers: Modifiers,
        capture: Option<NodeId>,
        space: &PointerSpace<'_>,
    ) {
        // Outside presses dismiss overlays before the press is delivered.
        self.dismiss_outside(hit);
        let primary = button == MouseButton::Primary;
        let native_focus = hit.and_then(|h| {
            let region = primary
                .then(|| {
                    let pos = space.local(self, h);
                    let region = space.region(self, h, pos)?;
                    let target = kivact_wire::FocusTarget {
                        node: h,
                        region: Some(region),
                    };
                    self.is_focus_target(target).then_some(target)
                })
                .flatten();
            region.or_else(|| {
                self.ancestor_where(h, |n| n.focusable(self.editors[n.index.0].is_some()))
                    .map(|node| kivact_wire::FocusTarget { node, region: None })
            })
        });
        let focus = hit
            .and_then(|h| self.pointer_focus_destination(h, space))
            .or(native_focus);
        let press = hit.and_then(|h| {
            if let Some(region) = space.region(self, h, space.local(self, h)) {
                let node = self.node(h)?;
                let declared = node.region(region)?;
                if declared.disabled {
                    return None;
                }
                if self.region_activatable(h, region) {
                    return Some(FocusTarget {
                        node: h,
                        region: Some(region),
                    });
                }
            }
            self.ancestor_where(h, |n| n.activatable())
                .map(|node| FocusTarget { node, region: None })
        });
        // A press of any button focuses; a primary press on nothing
        // focusable blurs. A later button of a captured gesture keeps the
        // focus its first press gave.
        if capture.is_none() {
            if let Some(f) = focus {
                self.set_focus_visible(Some(f), false);
            } else if primary && self.press_blurs(hit) {
                self.set_focus(None);
            }
        }
        if primary {
            let chain = self.pointer.state.track(pointer).hover.clone();
            self.set_chain(pointer, true, chain);
            self.pointer.state.track_mut(pointer).press_target = press;
        }
        let input = native_focus
            .filter(|f| f.region.is_none())
            .map(|f| f.node)
            .filter(|f| {
                self.tree.get(*f).is_some_and(|n| {
                    self.editors[n.index.0]
                        .as_deref()
                        .map(|editor| editor.editor.state())
                        .is_some()
                })
            });
        if let (Some(i), true) = (input, primary)
            && let Some(at) = space.text_offset(self, i)
        {
            self.place_caret(i, at, modifiers.shift);
        }
    }

    /// A button of `pointer` released over `hit` (`None`: nothing, or
    /// outside the window). `remaining` represents any buttons still held
    /// by this pointer. Each Up reaches the original listener; the final one
    /// releases the capture.
    #[expect(clippy::too_many_arguments, reason = "one release's fields")]
    pub(crate) fn pointer_up(
        &mut self,
        pointer: PointerId,
        hit: Option<NodeId>,
        button: MouseButton,
        remaining: Option<MouseButton>,
        modifiers: Modifiers,
        click_count: u8,
        space: &PointerSpace<'_>,
    ) {
        let hit = self.scoped(hit);
        let hit_chain = self.tree.chain(hit);
        let capture = self.pointer.state.track(pointer).capture;
        if remaining.is_none() {
            self.pointer.state.track_mut(pointer).capture = None;
        }
        let press = if button == MouseButton::Primary {
            self.set_chain(pointer, true, Vec::new());
            self.pointer.state.track_mut(pointer).press_target.take()
        } else {
            None
        };
        if let Some(t) = capture {
            self.send_pointer(
                pointer,
                t,
                PointerPhase::Up,
                Some(button),
                modifiers,
                click_count,
                space,
            );
        }
        if let Some(a) = press.filter(|a| hit_chain.contains(&a.node))
            && a.region.is_none_or(|region| {
                self.region_activatable(a.node, region)
                    && space.region(self, a.node, space.local(self, a.node)) == Some(region)
            })
        {
            self.push(HostEvent::Activate {
                node: a.node,
                region: a.region,
                pointer: Some(pointer),
            });
        }
    }

    /// Whether a currently published region offers enabled, visible activation.
    fn region_activatable(&self, node: NodeId, region: RegionId) -> bool {
        self.tree.shown(node)
            && self.node(node).is_some_and(|node| {
                !node.disabled()
                    && node.region(region).is_some_and(|region| {
                        !region.disabled
                            && !region.hidden
                            && region.actions.contains(&A11yAction::Activate)
                    })
            })
    }

    /// `pointer` moved over `hit` with `pressed` held (`None`: no button).
    pub(crate) fn pointer_move(
        &mut self,
        pointer: PointerId,
        hit: Option<NodeId>,
        pressed: Option<MouseButton>,
        modifiers: Modifiers,
        space: &PointerSpace<'_>,
    ) {
        // No button held while a capture is active: the release happened
        // where the host could not see it. End the drag with `Cancel`.
        if pressed.is_none() && self.pointer.state.track(pointer).capture.is_some() {
            self.cancel_pointer_input(pointer, space);
        }
        let hit = self.scoped(hit);
        let before = self.pointer.state.track(pointer).over;
        self.track_hover(pointer, hit, modifiers, space);
        let track = self.pointer.state.track(pointer);
        if let Some(t) = track.capture.or(track.over.filter(|o| before == Some(*o))) {
            self.send_pointer(pointer, t, PointerPhase::Move, pressed, modifiers, 0, space);
        }
        // A primary drag that started in the focused text input selects.
        let selecting = self
            .focused()
            .filter(|f| f.region.is_none())
            .map(|f| f.node)
            .filter(|f| {
                pressed == Some(MouseButton::Primary)
                    && self.tree.get(*f).is_some_and(|n| {
                        self.editors[n.index.0]
                            .as_deref()
                            .map(|editor| editor.editor.state())
                            .is_some()
                    })
                    && self.pointer.state.track(pointer).pressed.contains(f)
            });
        if let Some(i) = selecting
            && let Some(at) = space.text_offset(self, i)
        {
            self.place_caret(i, at, true);
        }
    }

    /// `pointer`'s capture is lost (the window was deactivated while
    /// pressed, the platform canceled the contact): `Cancel` to its
    /// capturing listener, and its press activates nothing.
    pub(crate) fn cancel_pointer_input(&mut self, pointer: PointerId, space: &PointerSpace<'_>) {
        // The `Cancel` and the `Leave` after it hold no button; the
        // pointer's next event reports what is held again.
        self.pointer.state.track_mut(pointer).source.buttons = PointerButtons::NONE;
        self.pointer.state.track_mut(pointer).touch_scroll = None;
        self.pointer.state.track_mut(pointer).touch_cancel_pending = false;
        self.set_chain(pointer, true, Vec::new());
        let track = self.pointer.state.track_mut(pointer);
        track.press_target = None;
        if let Some(t) = track.capture.take() {
            self.send_pointer(
                pointer,
                t,
                PointerPhase::Cancel,
                None,
                Modifiers::default(),
                0,
                space,
            );
        }
    }

    /// Disabling a capturing listener ends the drags it captured (each
    /// where its pointer last was), including component cleanup, before a
    /// later re-enable can resume the old gesture.
    pub(crate) fn cancel_disabled_capture(&mut self, engine: &mut arvane::Engine) {
        let disabled: Vec<(PointerId, Point)> = self
            .pointer
            .state
            .tracks
            .values()
            .filter(|t| {
                t.capture
                    .is_some_and(|id| self.tree.get(id).is_some_and(|node| node.disabled()))
            })
            .map(|t| (t.source.id, t.at))
            .collect();
        for (pointer, at) in disabled {
            let space = PointerSpace::new(at, engine);
            self.cancel_pointer_input(pointer, &space);
        }
    }

    /// Wheel over `hit`. Scroll containers below the nearest wheel listener
    /// scroll natively (edges propagate upward); once the delta reaches
    /// the listener the host stops and sends `Wheel`, and the core answers
    /// with `ScrollBy` unless a handler took it.
    pub(crate) fn handle_wheel(
        &mut self,
        hit: Option<NodeId>,
        delta: Point,
        modifiers: Modifiers,
        space: &PointerSpace<'_>,
    ) {
        let Some(hit) = self.scoped(hit) else { return };
        let listener = self.ancestor_where(hit, |n| n.listens_wheel());
        if self.scroll_from(hit, delta, listener) {
            return;
        }
        if let Some(t) = listener {
            let pos = space.local(self, t);
            let region = space.region(self, t, pos);
            self.push(HostEvent::Wheel {
                node: t,
                region,
                delta,
                pos,
                modifiers,
            });
        }
    }
}
