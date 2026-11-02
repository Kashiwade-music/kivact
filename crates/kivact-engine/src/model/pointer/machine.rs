//! Role: The pointer state machine: chains, press, release, move, cancel and wheel.

use super::space::PointerSpace;
use super::state::{PointerSource, Track};
use crate::model::Model;
use kivact_wire::{
    A11yAction, FocusTarget, HostEvent, Modifiers, MouseButton, NodeId, Point, PointerButtons,
    PointerEvent, PointerId, PointerPhase, RegionId,
};

impl Model {
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
    pub(super) fn begin_pointer(&mut self, source: PointerSource, at: Point) {
        let mut track = Track::new(source);
        track.at = at;
        self.pointer.state.tracks.insert(source.id.0, track);
    }

    /// Stop following the contact `pointer` (it ended).
    pub(super) fn end_pointer(&mut self, pointer: PointerId) {
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
    pub(super) fn set_chain(&mut self, pointer: PointerId, pressed: bool, mut chain: Vec<NodeId>) {
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
                    self.changes.moved();
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
