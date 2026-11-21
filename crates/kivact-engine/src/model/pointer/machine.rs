//! Role: The pointer state machine: chains, press, release, move, cancel and wheel.

use super::space::PointerSpace;
use super::state::{Capture, Press, SelectUnit, Track};
use crate::contract::HostEvent;
use crate::model::Model;
use kivact_types::{
    A11yAction, ButtonPress, FocusTarget, Modifiers, MouseButton, NodeId, Point, PointerButtons,
    PointerId, PointerPhase, PointerSample, RegionId, StyleState,
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

    /// Send `input`'s pointer's event `phase` to `node`, at the window
    /// position `at`; the sample's buttons, pressure, tilt, time and
    /// modifiers are `input`'s.
    fn send_pointer(
        &mut self,
        input: &PointerSample,
        node: NodeId,
        phase: PointerPhase,
        at: Point,
    ) {
        let event = self.pointer_sample(input, node, phase, at);
        self.push(HostEvent::Pointer(event));
    }

    fn pointer_sample(
        &self,
        input: &PointerSample,
        node: NodeId,
        phase: PointerPhase,
        at: Point,
    ) -> crate::contract::Routed<PointerSample> {
        let pos = self.pointer_local(node, at);
        let region = self.node(node).and_then(|n| n.region_at(pos));
        let is_primary = self.pointer.state.track(input.id).is_primary();
        crate::contract::Routed {
            window: self.window,
            target: Some(crate::contract::RouteTarget { node, region }),
            local: pos,
            activation: None,
            origin: self.frame.origin(),
            input: PointerSample {
                phase,
                position: at,
                buttons: if phase == PointerPhase::Cancel {
                    PointerButtons::NONE
                } else {
                    input.buttons
                },
                button: phase.press().map(|press| press.button),
                press: phase.press(),
                is_primary,
                ..*input
            },
        }
    }

    /// The sample a cancel without a shell sample carries: no button, no
    /// modifier, no pressure or tilt, at the host time of the event being
    /// handled.
    pub(super) fn cancel_sample(&self, pointer: PointerId, at: Point) -> PointerSample {
        let track = self.pointer.state.track(pointer);
        PointerSample {
            id: pointer,
            kind: track.kind(),
            phase: PointerPhase::Cancel,
            position: at,
            buttons: PointerButtons::NONE,
            button: None,
            press: None,
            pressure: None,
            tilt: None,
            time_ns: self.frame.host_now(),
            modifiers: Modifiers::default(),
            is_primary: track.is_primary(),
        }
    }

    /// Follow a new contact, `input`'s pointer, at the window position `at`;
    /// `input.is_primary` is the platform's `isPrimary` at its `Down`.
    pub(super) fn begin_pointer(&mut self, input: &PointerSample, at: Point) {
        let mut track = Track::contact(input.id, input.kind, input.is_primary);
        track.set_at(at);
        self.pointer.state.tracks.insert(input.id.0, track);
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
            self.group_dependents(&touched, |s| s.wants(StyleState::GroupHover))
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
            self.report_states(id);
            let has = self.tree.get(id).and_then(|n| n.states()).is_some_and(|s| {
                (hover && s.wants(StyleState::Hover)) || (press && s.wants(StyleState::Active))
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
            .map(|t| t.id())
            .collect();
        for pointer in pointers {
            self.clear_hover(pointer);
        }
    }

    /// `input`'s pointer's hover chain (a primary pointer's) and
    /// `Enter`/`Leave` of the listener under `hit`.
    fn track_hover(
        &mut self,
        input: &PointerSample,
        hit: Option<NodeId>,
        space: &PointerSpace<'_>,
    ) {
        let pointer = input.id;
        if self.pointer.state.track(pointer).is_primary() {
            let chain = self.tree.chain(hit);
            self.set_chain(pointer, Chain::Hover, chain);
        }
        let over = hit.and_then(|h| self.ancestor_where(h, |n| n.listens_pointer()));
        let old = self.pointer.state.track(pointer).over();
        self.pointer.state.track_mut(pointer).set_over(over);
        self.report_hover();
        if over != old {
            if let Some(o) = old.filter(|o| self.tree.contains(*o)) {
                self.send_pointer(input, o, PointerPhase::Leave, space.window());
            }
            if let Some(n) = over {
                self.send_pointer(input, n, PointerPhase::Enter, space.window());
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

    /// A button of `input`'s pointer pressed over `hit` (the host's hit test).
    pub(crate) fn pointer_down(
        &mut self,
        input: &PointerSample,
        hit: Option<NodeId>,
        space: &PointerSpace<'_>,
    ) {
        let PointerPhase::Down(press) = input.phase else {
            unreachable!("invariant: a press is a `Down`")
        };
        let pointer = input.id;
        let hit = self.scoped(hit);
        self.track_hover(input, hit, space);
        let capture = self.pointer.state.track(pointer).capture();
        if self.pointer.state.track(pointer).is_primary() {
            self.primary_press(pointer, hit, press, input.modifiers, capture, space);
        }
        let target =
            capture.or_else(|| hit.and_then(|h| self.ancestor_where(h, |n| n.listens_pointer())));
        if let Some(t) = target.filter(|t| self.tree.get(*t).is_some_and(|n| !n.disabled())) {
            let route = self.pointer.state.track_mut(pointer).route_mut();
            if route.capture.and_then(Capture::target).is_none() {
                route.capture = Some(Capture::Implicit(t));
            }
            self.send_pointer(input, t, PointerPhase::Down(press), space.window());
        }
    }

    /// What a primary pointer's press over `hit` does besides its `Down`:
    /// dismiss the overlays it lands outside of, focus (or blur), retain
    /// the pressed chain and the activation target, and place a text
    /// input's caret, or select its word or line for a double or triple
    /// click. `capture` is the pointer's capture before the press.
    fn primary_press(
        &mut self,
        pointer: PointerId,
        hit: Option<NodeId>,
        button_press: ButtonPress,
        modifiers: Modifiers,
        capture: Option<NodeId>,
        space: &PointerSpace<'_>,
    ) {
        // Outside presses dismiss overlays before the press is delivered.
        self.dismiss_outside(hit);
        let primary = button_press.button == MouseButton::Primary;
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
                    .is_some_and(|n| self.slots.editors[n.index.0].is_some())
            });
        let mut unit = None;
        if let (Some(i), true) = (input, primary)
            && let Some(at) = space.text_offset(self, i)
        {
            unit = Some(self.press_text_input(i, at, button_press.clicks, modifiers.shift));
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
                        .is_some_and(|n| self.slots.editors[n.index.0].is_some())
            });
        if let Some(editor) = selecting {
            let unit = unit.unwrap_or(SelectUnit::Char);
            self.pointer
                .state
                .track_mut(pointer)
                .start_selecting(editor, unit);
        }
    }

    /// A primary press over the text input `input` at byte offset `at`
    /// (`clicks`: its click count): a click places the caret (Shift
    /// extends), a double click selects the word there, a triple click the
    /// line. Returns what a drag from it extends by.
    fn press_text_input(
        &mut self,
        input: NodeId,
        at: usize,
        clicks: u8,
        extend: bool,
    ) -> SelectUnit {
        let multiline = self.tree.get(input).is_some_and(|n| n.multiline());
        let mut unit = SelectUnit::Char;
        match clicks {
            0 | 1 => self.place_caret(input, at, extend),
            _ => {
                self.edit_input(input, |e| {
                    unit = if clicks == 2 {
                        SelectUnit::Word(e.word_at(at))
                    } else {
                        SelectUnit::Line(e.line_at(at, multiline))
                    };
                    if let SelectUnit::Word(range) | SelectUnit::Line(range) = &unit {
                        e.select(range.clone());
                    }
                });
            }
        }
        unit
    }

    /// Extend the selection of `input` to the pointer at byte offset `at`
    /// by `unit`: a word or line drag keeps the clicked unit selected and
    /// reaches the end of the unit under the pointer.
    fn extend_selection(&mut self, input: NodeId, at: usize, unit: &SelectUnit) {
        let multiline = self.tree.get(input).is_some_and(|n| n.multiline());
        let origin = match unit {
            SelectUnit::Char => return self.place_caret(input, at, true),
            SelectUnit::Word(origin) | SelectUnit::Line(origin) => origin.clone(),
        };
        let words = matches!(unit, SelectUnit::Word(_));
        self.edit_input(input, |e| {
            let here = if words {
                e.word_at(at)
            } else {
                e.line_at(at, multiline)
            };
            if here.start < origin.start {
                e.select(origin.end..here.start);
            } else {
                e.select(origin.start..here.end.max(origin.end));
            }
        });
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
                self.ancestor_where(h, |n| n.focusable(self.slots.editors[n.index.0].is_some()))
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
        input: &PointerSample,
        hit: Option<NodeId>,
        space: &PointerSpace<'_>,
    ) {
        let PointerPhase::Up(press) = input.phase else {
            unreachable!("invariant: a release is an `Up`")
        };
        let pointer = input.id;
        let hit = self.scoped(hit);
        let hit_chain = self.tree.chain(hit);
        let capture = self.pointer.state.track(pointer).capture();
        let hit_tested = self
            .pointer
            .state
            .track(pointer)
            .route()
            .is_some_and(|route| route.capture == Some(Capture::Released));
        let released = input.buttons.is_empty();
        let pressed = if press.button == MouseButton::Primary {
            self.set_chain(pointer, Chain::Pressed, Vec::new());
            let track = self.pointer.state.track_mut(pointer);
            let pressed = track
                .routed_mut()
                .and_then(|route| route.press.take())
                .and_then(|press| press.target);
            track.stop_selecting();
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
            self.send_pointer(input, t, PointerPhase::Up(press), space.window());
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
                    input,
                    a.node,
                    PointerPhase::Up(press),
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

    /// `input`'s pointer moved over `hit`, with the buttons `input` holds.
    pub(crate) fn pointer_move(
        &mut self,
        input: &PointerSample,
        hit: Option<NodeId>,
        space: &PointerSpace<'_>,
    ) {
        let pointer = input.id;
        let held = input.buttons;
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
        self.track_hover(input, hit, space);
        let track = self.pointer.state.track(pointer);
        if let Some(t) = track
            .capture()
            .or(track.over().filter(|o| before == Some(*o)))
        {
            self.send_pointer(input, t, PointerPhase::Move, space.window());
        }
        // A primary drag that started in the focused text input selects.
        let selecting = self
            .pointer
            .state
            .track(pointer)
            .selecting()
            .filter(|(editor, _)| {
                held.contains(MouseButton::Primary)
                    && self
                        .focused()
                        .is_some_and(|f| f.node == *editor && f.region.is_none())
            })
            .map(|(editor, unit)| (editor, unit.clone()));
        if let Some((editor, unit)) = selecting
            && let Some(at) = space.text_offset(self, editor)
        {
            self.extend_selection(editor, at, &unit);
        }
    }

    /// `pointer`'s capture is lost (the window was deactivated while
    /// pressed, the platform canceled the contact): `Cancel` to its
    /// capturing listener, and its press activates nothing. It needs no
    /// text engine, so it is delivered at once.
    pub(crate) fn cancel_pointer_input(&mut self, pointer: PointerId) {
        let at = self.pointer.state.track(pointer).at();
        let sample = self.cancel_sample(pointer, at);
        self.cancel_pointer_at(pointer, at, &sample);
    }

    /// Cancel `pointer`'s gesture, sending the `Cancel` as `input` (a sample
    /// that holds no button: the `Cancel` and the `Leave` after it hold none;
    /// the pointer's next event reports what is held again).
    pub(super) fn cancel_pointer_at(
        &mut self,
        pointer: PointerId,
        at: Point,
        input: &PointerSample,
    ) {
        self.set_chain(pointer, Chain::Pressed, Vec::new());
        let track = self.pointer.state.track_mut(pointer);
        if let Some(t) = track.reset_gesture() {
            self.send_pointer(input, t, PointerPhase::Cancel, at);
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
            .map(|t| t.id())
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
                target: Some(crate::contract::RouteTarget { node: t, region }),
                local: pos,
                activation: None,
                origin: self.frame.origin(),
                input: kivact_types::WheelInput {
                    delta,
                    position: space.window(),
                    modifiers,
                },
            }));
        }
    }
}
