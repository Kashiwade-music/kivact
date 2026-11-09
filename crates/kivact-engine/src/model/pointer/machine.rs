//! Role: The pointer state machine: chains, press, release, move, cancel and wheel.

use super::space::PointerSpace;
use super::state::{Capture, MouseGesture, PointerSource, Press, Track};
use crate::contract::HostEvent;
use crate::model::Model;
use kivact_types::{
    A11yAction, ButtonPress, FocusTarget, Modifiers, MouseButton, NodeId, Point, PointerButtons,
    PointerId, PointerPhase, PointerSample, RegionId,
};

/// Which of a pointer's two chains a change is about.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Chain {
    /// The hit node and its ancestors.
    Hover,
    /// The hover chain at a primary press, until the release.
    Pressed,
}

impl Model {
    fn report_hover(&mut self) {
        let target = self.pointer.state.tracks.values().find_map(|track| {
            let node = track.chain().first().copied()?;
            let local = self.pointer_local(node, track.at());
            Some(FocusTarget {
                node,
                region: self.node(node).and_then(|n| n.region_at(local)),
            })
        });
        if self.pointer.hover_sent.update(target).is_some() {
            self.push(HostEvent::HoverChanged {
                window: self.window,
                target,
            });
        }
    }

    fn send_pointer(
        &mut self,
        pointer: PointerId,
        node: NodeId,
        phase: PointerPhase,
        modifiers: Modifiers,
        at: Point,
    ) {
        let event = self.pointer_sample(pointer, node, phase, modifiers, at);
        self.push(HostEvent::Pointer(event));
    }

    fn pointer_sample(
        &self,
        pointer: PointerId,
        node: NodeId,
        phase: PointerPhase,
        modifiers: Modifiers,
        at: Point,
    ) -> crate::contract::Routed<PointerSample> {
        let pos = self.pointer_local(node, at);
        let region = self.node(node).and_then(|n| n.region_at(pos));
        let source = *self.pointer.state.track(pointer).source();
        crate::contract::Routed {
            window: self.window,
            target: Some(node),
            region,
            local: pos,
            captured: matches!(phase, PointerPhase::Up(_) | PointerPhase::Cancel)
                || self.pointer.state.track(pointer).capture() == Some(node),
            activation: None,
            input: PointerSample {
                id: source.id,
                phase,
                position: at,
                buttons: if phase == PointerPhase::Cancel {
                    PointerButtons::NONE
                } else {
                    source.buttons
                },
                button: phase.press().map(|press| press.button),
                press: phase.press(),
                modifiers,
                kind: source.kind,
                pressure: source.pressure,
                tilt: source.tilt,
                is_primary: source.primary,
                time_ns: source.time_ns,
            },
        }
    }

    /// Follow a new contact, `source`, at the window position `at`.
    pub(super) fn begin_pointer(&mut self, source: PointerSource, at: Point) {
        let mut track = Track::new(source);
        track.set_at(at);
        self.pointer.state.tracks.insert(source.id.0, track);
    }

    /// Stop following the contact `pointer` (it ended).
    pub(super) fn end_pointer(&mut self, pointer: PointerId) {
        self.pointer.state.tracks.remove(&pointer.0);
    }

    /// Swap `pointer`'s pressed or hover chain with `chain`.
    fn swap_chain(&mut self, pointer: PointerId, which: Chain, chain: &mut Vec<NodeId>) {
        let track = self.pointer.state.track_mut(pointer);
        let own = match which {
            Chain::Hover => {
                let Some(hover) = track.hover_mut() else {
                    return;
                };
                &mut hover.chain
            }
            Chain::Pressed => {
                let Some(press) = track.routed_mut().and_then(|route| route.press.as_mut()) else {
                    assert!(
                        chain.is_empty(),
                        "invariant: a pressed chain is set only during a press"
                    );
                    return;
                };
                &mut press.chain
            }
        };
        std::mem::swap(own, chain);
    }

    /// Replace `pointer`'s pressed or hover chain, repainting the nodes
    /// whose shown hover or pressed state flips.
    fn set_chain(&mut self, pointer: PointerId, which: Chain, mut chain: Vec<NodeId>) {
        let track = self.pointer.state.track(pointer);
        let old = match which {
            Chain::Hover => track.chain(),
            Chain::Pressed => track.pressed(),
        };
        if old == chain.as_slice() {
            return;
        }
        let touched: Vec<NodeId> = old
            .iter()
            .filter(|x| !chain.contains(x))
            .chain(chain.iter().filter(|x| !old.contains(x)))
            .copied()
            .collect();
        // The hover of a group restyles the nodes below it.
        let dependents = if which == Chain::Pressed {
            Vec::new()
        } else {
            self.group_dependents(&touched, |s| s.group_hover.is_some())
        };
        let shown = |model: &Model, id: NodeId| (model.is_hovered(id), model.is_pressed(id));
        let before: Vec<_> = touched.iter().map(|id| shown(self, *id)).collect();
        self.swap_chain(pointer, which, &mut chain);
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
            self.swap_chain(pointer, which, &mut chain);
            for id in flipped
                .iter()
                .map(|(id, ..)| *id)
                .chain(dependents.iter().copied())
            {
                self.prepare_state_motion(id);
            }
            self.swap_chain(pointer, which, &mut chain);
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
                    self.changes
                        .layout
                        .insert(crate::model::changes::LayoutReasons::PASS);
                    self.changes.moved();
                }
            }
        }
        self.report_hover();
    }

    /// `pointer` hovers nothing, keeping its capture and the listener it
    /// entered (the mouse left the window with a button held, the hit was
    /// removed).
    pub(crate) fn clear_hover(&mut self, pointer: PointerId) {
        self.set_chain(pointer, Chain::Hover, Vec::new());
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
            .filter(|t| t.chain().first().is_some_and(|hit| stale(self, *hit)))
            .map(|t| t.source().id)
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
        if self.pointer.state.track(pointer).source().primary {
            let chain = self.tree.chain(hit);
            self.set_chain(pointer, Chain::Hover, chain);
        }
        let over = hit.and_then(|h| self.ancestor_where(h, |n| n.listens_pointer()));
        let old = self.pointer.state.track(pointer).over();
        self.pointer.state.track_mut(pointer).set_over(over);
        self.report_hover();
        if over != old {
            if let Some(o) = old.filter(|o| self.tree.contains(*o)) {
                self.send_pointer(pointer, o, PointerPhase::Leave, modifiers, space.window());
            }
            if let Some(n) = over {
                self.send_pointer(pointer, n, PointerPhase::Enter, modifiers, space.window());
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
        press: ButtonPress,
        modifiers: Modifiers,
        space: &PointerSpace<'_>,
    ) {
        let hit = self.scoped(hit);
        self.track_hover(pointer, hit, modifiers, space);
        let capture = self.pointer.state.track(pointer).capture();
        if self.pointer.state.track(pointer).source().primary {
            self.primary_press(pointer, hit, press.button, modifiers, capture, space);
        }
        let target =
            capture.or_else(|| hit.and_then(|h| self.ancestor_where(h, |n| n.listens_pointer())));
        if let Some(t) = target.filter(|t| self.tree.get(*t).is_some_and(|n| !n.disabled())) {
            let route = self.pointer.state.track_mut(pointer).route_mut();
            if route.capture.and_then(Capture::target).is_none() {
                route.capture = Some(Capture::Implicit(t));
            }
            self.send_pointer(
                pointer,
                t,
                PointerPhase::Down(press),
                modifiers,
                space.window(),
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
        let native_focus = self.press_native_focus(hit, primary, space);
        let focus = hit
            .and_then(|h| self.pointer_focus_destination(h, space))
            .or(native_focus);
        let press = self.press_activation_target(hit, space);
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
            let chain = self.pointer.state.track(pointer).chain().to_vec();
            let route = self.pointer.state.track_mut(pointer).route_mut();
            if route.press.is_none() {
                route.press = Some(Press {
                    chain: Vec::new(),
                    target: None,
                });
            }
            self.set_chain(pointer, Chain::Pressed, chain);
            self.pointer
                .state
                .track_mut(pointer)
                .route_mut()
                .press
                .as_mut()
                .expect("invariant: the press was just made")
                .target = press;
        }
        let input = native_focus
            .filter(|f| f.region.is_none())
            .map(|f| f.node)
            .filter(|f| {
                self.tree
                    .get(*f)
                    .is_some_and(|n| self.editors[n.index.0].is_some())
            });
        if let (Some(i), true) = (input, primary)
            && let Some(at) = space.text_offset(self, i)
        {
            self.place_caret(i, at, modifiers.shift);
        }
        let selecting = self
            .focused()
            .filter(|f| f.region.is_none())
            .map(|f| f.node)
            .filter(|f| {
                primary
                    && self.pointer.state.track(pointer).pressed().contains(f)
                    && self
                        .tree
                        .get(*f)
                        .is_some_and(|n| self.editors[n.index.0].is_some())
            });
        if let Some(editor) = selecting
            && let Track::Mouse(mouse) = self.pointer.state.track_mut(pointer)
            && let MouseGesture::Route(route) = &mut mouse.gesture
        {
            mouse.gesture = MouseGesture::SelectText {
                editor,
                route: std::mem::take(route),
            };
        }
    }

    fn press_activation_target(
        &self,
        hit: Option<NodeId>,
        space: &PointerSpace<'_>,
    ) -> Option<FocusTarget> {
        hit.and_then(|h| {
            if let Some(region) = space.region(self, h, space.local(self, h)) {
                let node = self.node(h)?;
                let declared = node.region(region)?;
                if declared.semantics.disabled {
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
        })
    }

    fn press_native_focus(
        &self,
        hit: Option<NodeId>,
        primary: bool,
        space: &PointerSpace<'_>,
    ) -> Option<FocusTarget> {
        hit.and_then(|h| {
            let region = primary
                .then(|| {
                    let pos = space.local(self, h);
                    let region = space.region(self, h, pos)?;
                    let target = kivact_types::FocusTarget {
                        node: h,
                        region: Some(region),
                    };
                    self.is_focus_target(target).then_some(target)
                })
                .flatten();
            region.or_else(|| {
                self.ancestor_where(h, |n| n.focusable(self.editors[n.index.0].is_some()))
                    .map(|node| kivact_types::FocusTarget { node, region: None })
            })
        })
    }

    /// A button of `pointer` released over `hit` (`None`: nothing, or
    /// outside the window). Each Up reaches the capturing listener, or the
    /// enabled listener under the pointer once a listener released its
    /// capture (none after a `Cancel`); the one that leaves no button held
    /// releases the capture.
    pub(crate) fn pointer_up(
        &mut self,
        pointer: PointerId,
        hit: Option<NodeId>,
        press: ButtonPress,
        modifiers: Modifiers,
        space: &PointerSpace<'_>,
    ) {
        let hit = self.scoped(hit);
        let hit_chain = self.tree.chain(hit);
        let capture = self.pointer.state.track(pointer).capture();
        let hit_tested = self
            .pointer
            .state
            .track(pointer)
            .route()
            .is_some_and(|route| route.capture == Some(Capture::Released));
        let released = self
            .pointer
            .state
            .track(pointer)
            .source()
            .buttons
            .is_empty();
        let pressed = if press.button == MouseButton::Primary {
            self.set_chain(pointer, Chain::Pressed, Vec::new());
            let track = self.pointer.state.track_mut(pointer);
            let pressed = track
                .routed_mut()
                .and_then(|route| route.press.take())
                .and_then(|press| press.target);
            if let Track::Mouse(mouse) = track
                && let MouseGesture::SelectText { route, .. } = &mut mouse.gesture
            {
                mouse.gesture = MouseGesture::Route(std::mem::take(route));
            }
            pressed
        } else {
            None
        };
        if released && let Some(route) = self.pointer.state.track_mut(pointer).routed_mut() {
            route.capture = None;
        }
        let listener = || {
            hit.and_then(|h| self.ancestor_where(h, |n| n.listens_pointer()))
                .filter(|t| self.tree.get(*t).is_some_and(|n| !n.disabled()))
        };
        if let Some(t) = capture.or_else(|| hit_tested.then(listener).flatten()) {
            self.send_pointer(
                pointer,
                t,
                PointerPhase::Up(press),
                modifiers,
                space.window(),
            );
        }
        if let Some(a) = pressed.filter(|a| hit_chain.contains(&a.node))
            && a.region.is_none_or(|region| {
                self.region_activatable(a.node, region)
                    && space.region(self, a.node, space.local(self, a.node)) == Some(region)
            })
        {
            self.push(HostEvent::Activate {
                node: a.node,
                region: a.region,
                pointer: Some(self.pointer_sample(
                    pointer,
                    a.node,
                    PointerPhase::Up(press),
                    modifiers,
                    space.window(),
                )),
            });
        }
    }

    /// Whether a currently published region offers enabled, visible activation.
    fn region_activatable(&self, node: NodeId, region: RegionId) -> bool {
        self.tree.shown(node)
            && self.node(node).is_some_and(|node| {
                !node.disabled()
                    && node.region(region).is_some_and(|region| {
                        !region.semantics.disabled
                            && !region.semantics.hidden
                            && region
                                .semantics
                                .actions
                                .as_ref()
                                .is_some_and(|actions| actions.contains(&A11yAction::Activate))
                    })
            })
    }

    pub(crate) fn pointer_capture(&self, pointer: PointerId) -> Option<NodeId> {
        self.pointer
            .state
            .tracks
            .get(&pointer.0)
            .and_then(Track::capture)
    }

    /// Explicit capture shares the gesture route with implicit press capture.
    pub(crate) fn set_pointer_capture(&mut self, node: NodeId, pointer: PointerId, capture: bool) {
        if (capture && !self.in_scope(node))
            || !self.tree.live(node)
            || self.tree.get(node).is_none_or(|n| n.disabled())
        {
            return;
        }
        if capture {
            self.reserve_touch_gesture(pointer);
        }
        if let Some(track) = self.pointer.state.tracks.get_mut(&pointer.0)
            && let Some(route) = track.routed_mut()
        {
            if capture {
                route.capture = Some(Capture::Explicit(node));
            } else if route
                .capture
                .is_some_and(|capture| capture.target() == Some(node))
            {
                route.capture = Some(Capture::Released);
            }
        }
    }

    /// `pointer` moved over `hit`, with the buttons its source holds.
    pub(crate) fn pointer_move(
        &mut self,
        pointer: PointerId,
        hit: Option<NodeId>,
        modifiers: Modifiers,
        space: &PointerSpace<'_>,
    ) {
        let held = self.pointer.state.track(pointer).source().buttons;
        // No button held while a capture is active: the release happened
        // where the host could not see it. End the drag with `Cancel`.
        if held.is_empty()
            && self
                .pointer
                .state
                .track(pointer)
                .route()
                .is_some_and(|route| matches!(route.capture, Some(Capture::Implicit(_))))
        {
            self.cancel_pointer_input(pointer);
        }
        let hit = self.scoped(hit);
        let before = self.pointer.state.track(pointer).over();
        self.track_hover(pointer, hit, modifiers, space);
        let track = self.pointer.state.track(pointer);
        if let Some(t) = track
            .capture()
            .or(track.over().filter(|o| before == Some(*o)))
        {
            self.send_pointer(pointer, t, PointerPhase::Move, modifiers, space.window());
        }
        // A primary drag that started in the focused text input selects.
        let selecting = match self.pointer.state.track(pointer) {
            Track::Mouse(mouse) => match mouse.gesture {
                MouseGesture::SelectText { editor, .. }
                    if held.contains(MouseButton::Primary)
                        && self
                            .focused()
                            .is_some_and(|f| f.node == editor && f.region.is_none()) =>
                {
                    Some(editor)
                }
                _ => None,
            },
            track => self
                .focused()
                .filter(|f| f.region.is_none())
                .map(|f| f.node)
                .filter(|f| {
                    held.contains(MouseButton::Primary)
                        && track.pressed().contains(f)
                        && self
                            .tree
                            .get(*f)
                            .is_some_and(|n| self.editors[n.index.0].is_some())
                }),
        };
        if let Some(i) = selecting
            && let Some(at) = space.text_offset(self, i)
        {
            self.place_caret(i, at, true);
        }
    }

    /// `pointer`'s capture is lost (the window was deactivated while
    /// pressed, the platform canceled the contact): `Cancel` to its
    /// capturing listener, and its press activates nothing. It needs no
    /// text engine, so it is delivered at once.
    pub(crate) fn cancel_pointer_input(&mut self, pointer: PointerId) {
        let at = self.pointer.state.track(pointer).at();
        self.cancel_pointer_at(pointer, at);
    }

    pub(super) fn cancel_pointer_at(&mut self, pointer: PointerId, at: Point) {
        // The `Cancel` and the `Leave` after it hold no button; the
        // pointer's next event reports what is held again.
        self.pointer.state.track_mut(pointer).source_mut().buttons = PointerButtons::NONE;
        self.set_chain(pointer, Chain::Pressed, Vec::new());
        let track = self.pointer.state.track_mut(pointer);
        if let Some(t) = track.reset_gesture() {
            self.send_pointer(pointer, t, PointerPhase::Cancel, Modifiers::default(), at);
        }
    }

    /// Disabling a capturing listener ends the drags it captured (each
    /// where its pointer last was), including component cleanup, before a
    /// later re-enable can resume the old gesture.
    pub(crate) fn cancel_disabled_capture(&mut self) {
        let disabled: Vec<PointerId> = self
            .pointer
            .state
            .tracks
            .values()
            .filter(|t| {
                t.capture()
                    .is_some_and(|id| self.tree.get(id).is_some_and(|node| node.disabled()))
            })
            .map(|t| t.source().id)
            .collect();
        for pointer in disabled {
            self.cancel_pointer_input(pointer);
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
            self.push(HostEvent::Wheel(crate::contract::Routed {
                window: self.window,
                target: Some(t),
                region,
                local: pos,
                captured: false,
                activation: None,
                input: kivact_types::WheelInput {
                    delta,
                    position: space.window(),
                    modifiers,
                },
            }));
        }
    }
}
