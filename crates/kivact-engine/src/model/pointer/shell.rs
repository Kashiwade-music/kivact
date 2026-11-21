//! Role: Translation of the shell's pointer input into the pointer state machine.
//!
//! Invariants:
//! - Hit testing is the model's (`Model::hit`) on the layout it
//!   computed; transforms do not move it. [`PointerSpace`] says where the
//!   pointer is relative to a node and which caret of a text input it is
//!   over (Arvane's shaping, `text_geometry`).
//! - A press carries its click count (Arvane counts it from the system's
//!   double-click settings).
//! - The mouse is always followed (`PointerId::MOUSE`); its moves, buttons
//!   and wheel go on whatever contacts are down. A press on a scrollbar makes
//!   the mouse's gesture a `MouseGesture::Scrollbar` until the primary release.
//! - A touch or pen contact is followed from its `Down` to its `Up` or
//!   `Cancel` as a pointer of its own (id, kind, the pressure of its last
//!   event, the tap count Arvane counted) that holds the primary button
//!   while it lasts. It is primary when no other contact of its kind is in
//!   contact at its `Down`, until it ends: when the primary finger lifts,
//!   the fingers still down stay secondary, and the next primary is the
//!   first finger down after all have lifted. It hovers (a primary contact)
//!   only while it lasts (entered before its `Down`, left after its `Up` or
//!   `Cancel`) and leaves what it entered when it ends, so a finger leaves
//!   no hover behind; the mouse's hover and `Enter`/`Leave` are its own and
//!   stay. A tap focuses and activates as a click does; a double tap has a
//!   click count of 2. A `Down` of an id still down first ends that
//!   contact as a `Cancel`.
//! - A `Cancel` ends the contact where it last was, without activating
//!   anything and without touching the other pointers. Losing the window's
//!   focus cancels every pointer.
//! - A file drag enters and leaves the nearest file listener under it (the
//!   hit or its nearest ancestor that listens); moving between nodes inside
//!   that listener sends neither, only `Over`. A drop ends it.

use super::state::{MouseGesture, ScrollDrag};
use crate::PointerSpace;
use crate::contract::{HostEvent, InputFlag};
use crate::model::Model;
use kivact_types::PointerSample;
use kivact_types::{
    ButtonPress, FilePhase, Modifiers, MouseButton, Point, PointerButtons, PointerId, PointerKind,
    PointerPhase,
};

impl Model {
    /// One pointer event of the window: the mouse's go to the mouse's
    /// state machine below, a contact's to [`Self::contact`]. Each listener
    /// event of the pointer carries the buttons, pressure, tilt and time of
    /// the sample it answers.
    pub(crate) fn pointer_input(
        &mut self,
        input: PointerSample,
        engine: &mut arvane_text::TextEngine,
    ) {
        self.report_modifiers(input.modifiers);
        // The mouse's id is the mouse's only; a window gives contacts others.
        if (input.kind == PointerKind::Mouse) != (input.id == PointerId::MOUSE) {
            return;
        }
        if input.kind != PointerKind::Mouse {
            return self.contact(input, engine);
        }
        match input.phase {
            PointerPhase::Move | PointerPhase::Enter => {
                self.mouse_moved(input.position, &input, engine)
            }
            PointerPhase::Leave => self.mouse_left(&input, engine),
            PointerPhase::Down(press) | PointerPhase::Up(press) => {
                // Where the press or release is; a release outside the
                // window does not bring the mouse back into it.
                self.pointer.state.mouse_mut().at = input.position;
                let pressed = matches!(input.phase, PointerPhase::Down(_));
                self.mouse_button(input.position, press, pressed, &input, engine);
            }
            // The window lost the mouse's gesture (its focus): it ends as
            // a `Cancel` where the mouse was, and nothing stays hovered.
            PointerPhase::Cancel => {
                // The `Cancel` and the `Leave` after it hold no button and
                // no modifier; the time, pressure and tilt are the window's.
                let ended = PointerSample {
                    buttons: PointerButtons::NONE,
                    modifiers: Modifiers::default(),
                    ..input
                };
                self.report_mouse_position(None);
                self.pointer.state.mouse_mut().at = input.position;
                self.cancel_pointer_at(PointerId::MOUSE, input.position, &ended);
                let space = PointerSpace::new(input.position, engine);
                self.pointer_move(&ended, None, &space);
                self.sync();
            }
        }
    }

    /// The mouse's window position at its last event.
    fn mouse_at(&self) -> Point {
        self.pointer.state.mouse().at
    }

    /// Report the mouse's window position (`None`: outside the window)
    /// when it differs from the last one reported.
    fn report_mouse_position(&mut self, position: Option<Point>) {
        if self.pointer.position_sent.update(position).is_some() {
            self.push(HostEvent::MousePosition {
                window: self.window,
                position,
            });
        }
    }

    /// The mouse moved to `at` (window coordinates), as `input`.
    fn mouse_moved(
        &mut self,
        at: Point,
        input: &PointerSample,
        engine: &mut arvane_text::TextEngine,
    ) {
        self.pointer.state.mouse_mut().at = at;
        self.report_mouse_position(Some(at));
        if let MouseGesture::Scrollbar(drag) = self.pointer.state.mouse().gesture {
            let local = self.pointer_local(drag.node, at);
            if let Some(bar) = self
                .scrollbar_bars(drag.node)
                .into_iter()
                .flatten()
                .find(|b| b.vertical == drag.vertical)
            {
                let pointer = if drag.vertical { local.y } else { local.x };
                let mut offset = self.tree.scroll_of(drag.node);
                let delta = if bar.travel > 0.0 {
                    (pointer - drag.pointer) * bar.range / bar.travel
                } else {
                    0.0
                };
                if drag.vertical {
                    offset.y = drag.offset + delta;
                } else {
                    offset.x = drag.offset + delta;
                }
                self.set_scroll(drag.node, offset);
                self.sync();
                self.settle(engine);
                return;
            }
            self.pointer.state.mouse_mut().gesture = MouseGesture::default();
        }
        let hit = self.hit(at);
        let space = PointerSpace::new(at, engine);
        self.pointer_move(input, hit, &space);
        self.sync();
    }

    /// The mouse left the window; a held button keeps its capture.
    fn mouse_left(&mut self, input: &PointerSample, engine: &mut arvane_text::TextEngine) {
        let at = self.mouse_at();
        self.report_mouse_position(None);
        if input.buttons.is_empty() {
            let space = PointerSpace::new(at, engine);
            self.pointer_move(input, None, &space);
        } else {
            // A held button keeps its capture and the listener it entered,
            // but the mouse is over nothing in the window.
            self.clear_hover(PointerId::MOUSE);
        }
        self.sync();
    }

    /// A mouse button pressed or released at the mouse's position, the
    /// `press.clicks`-th click of a multi-click (a release: its press's
    /// count).
    fn mouse_button(
        &mut self,
        at: Point,
        press: ButtonPress,
        pressed: bool,
        input: &PointerSample,
        engine: &mut arvane_text::TextEngine,
    ) {
        let button = press.button;
        if matches!(
            self.pointer.state.mouse().gesture,
            MouseGesture::Scrollbar(_)
        ) {
            if !pressed && button == MouseButton::Primary {
                self.pointer.state.mouse_mut().gesture = MouseGesture::default();
            }
            return;
        }
        let hit = self.hit(at);
        if pressed
            && button == MouseButton::Primary
            && self
                .pointer
                .state
                .track(PointerId::MOUSE)
                .capture()
                .is_none()
            && let Some(id) = hit
        {
            let local = self.pointer_local(id, at);
            if let Some(bar) = self.scrollbar_at(id, local) {
                let mut offset = self.tree.scroll_of(id);
                let pointer = if bar.vertical { local.y } else { local.x };
                if !bar.thumb.contains(local) {
                    let thumb = if bar.vertical {
                        bar.thumb.height
                    } else {
                        bar.thumb.width
                    };
                    let value = if bar.travel > 0.0 {
                        (pointer - thumb / 2.0) * bar.range / bar.travel
                    } else {
                        0.0
                    };
                    if bar.vertical {
                        offset.y = value;
                    } else {
                        offset.x = value;
                    }
                    self.set_scroll(id, offset);
                }
                let offset = self.tree.scroll_of(id);
                self.pointer.state.mouse_mut().gesture = MouseGesture::Scrollbar(ScrollDrag {
                    node: id,
                    vertical: bar.vertical,
                    pointer,
                    offset: if bar.vertical { offset.y } else { offset.x },
                });
                self.sync();
                self.settle(engine);
                return;
            }
        }
        let space = PointerSpace::new(at, engine);
        if pressed {
            self.pointer_down(input, hit, &space);
        } else {
            self.pointer_up(input, hit, &space);
        }
        self.sync();
    }

    /// Every pointer capture is lost (the window was deactivated): each
    /// pointer's drag ends with `Cancel` where that pointer last was,
    /// nothing stays hovered, and the contacts end.
    pub(crate) fn cancel_pointer(&mut self, engine: &mut arvane_text::TextEngine) {
        let pointers: Vec<(PointerId, Point)> = self
            .pointer
            .state
            .tracks
            .values()
            .map(|t| (t.id(), t.at()))
            .collect();
        self.report_mouse_position(None);
        for (pointer, at) in pointers {
            let sample = self.cancel_sample(pointer, at);
            self.cancel_pointer_at(pointer, at, &sample);
            let space = PointerSpace::new(at, engine);
            self.pointer_move(&sample, None, &space);
            if pointer != PointerId::MOUSE {
                self.end_pointer(pointer);
            }
        }
        self.sync();
    }

    /// A touch or pen contact began, moved or ended. Every contact is a
    /// pointer of its own that presses the primary button from its `Down`
    /// to its `Up` or `Cancel`, whatever the mouse and the other contacts
    /// do; the first of its kind is primary. It hovers only while it lasts,
    /// so it leaves no hover state behind, and a `Cancel` ends it where it
    /// last was without activating anything. A `Down` of an id that is still
    /// down cancels that contact first and then begins a new one.
    fn contact(&mut self, input: PointerSample, engine: &mut arvane_text::TextEngine) {
        let id = input.id;
        let at = input.position;
        let known = self.pointer.state.tracks.contains_key(&id.0);
        match input.phase {
            PointerPhase::Down(_) => {
                if id == PointerId::MOUSE {
                    return;
                }
                if known {
                    // A contact whose `Up` or `Cancel` never came: end it as a
                    // `Cancel` would, so the new contact starts clean.
                    self.cancel_contact(id, engine);
                }
                self.begin_pointer(&input, at);
                let hit = self.hit(at);
                let space = PointerSpace::new(at, engine);
                self.pointer_down(&input, hit, &space);
                if input.kind == PointerKind::Touch {
                    self.touch_candidate(&input, hit, at);
                }
            }
            _ if !known => return,
            PointerPhase::Move => {
                self.contact_at(id, at);
                if input.kind == PointerKind::Touch && self.touch_move(&input, at, engine) {
                    self.sync();
                    return;
                }
                let hit = self.hit(at);
                let space = PointerSpace::new(at, engine);
                self.pointer_move(&input, hit, &space);
            }
            PointerPhase::Up(_) => {
                self.contact_at(id, at);
                if input.kind == PointerKind::Touch {
                    self.release_touch_scroll(id, input.time_ns);
                }
                let hit = self.hit(at);
                let space = PointerSpace::new(at, engine);
                self.pointer_up(&input, hit, &space);
                self.end_contact(&input, &space);
            }
            // A window reports no hovering contact (`Contacts`).
            PointerPhase::Enter | PointerPhase::Leave => {}
            PointerPhase::Cancel => self.cancel_contact(id, engine),
        }
        self.sync();
    }

    /// The contact `id` is at `at`.
    fn contact_at(&mut self, id: PointerId, at: Point) {
        self.pointer.state.track_mut(id).set_at(at);
    }

    /// A contact ends as a `Cancel` would: it ends where it last was,
    /// without activating anything, and leaves what it entered.
    fn cancel_contact(&mut self, id: PointerId, engine: &mut arvane_text::TextEngine) {
        self.cancel_pointer_input(id);
        let at = self.pointer.state.track(id).at();
        let sample = self.cancel_sample(id, at);
        let space = PointerSpace::new(at, engine);
        self.end_contact(&sample, &space);
    }

    /// The contact `input`'s pointer ended: it leaves what it entered and
    /// hovered (a finger leaves no hover behind) and is followed no more.
    fn end_contact(&mut self, input: &PointerSample, space: &PointerSpace<'_>) {
        self.pointer_move(input, None, space);
        self.end_pointer(input.id);
    }

    /// Wheel at the mouse's position (`delta` in logical pixels).
    pub(crate) fn wheel(
        &mut self,
        input: kivact_types::WheelInput,
        engine: &mut arvane_text::TextEngine,
    ) {
        let delta = input.delta;
        self.report_modifiers(input.modifiers);
        let modifiers = input.modifiers;
        self.pointer.state.mouse_mut().at = input.position;
        let at = input.position;
        let hit = self.hit(at);
        let space = PointerSpace::new(at, engine);
        self.handle_wheel(hit, delta, modifiers, &space);
        self.sync();
    }

    /// A file drag over the window moved, dropped or left at `point`:
    /// the nearest file listener under it is entered, hovered or dropped
    /// on, and the one it left is told.
    pub(crate) fn file(
        &mut self,
        phase: kivact_types::FilePhase,
        files: Vec<std::path::PathBuf>,
        point: Point,
    ) {
        let hit = if phase == kivact_types::FilePhase::Leave {
            None
        } else {
            self.hit(point).and_then(|hit| {
                self.tree().chain(Some(hit)).into_iter().find(|id| {
                    self.node(*id).is_some_and(|n| {
                        n.properties
                            .input
                            .flags
                            .get(InputFlag::ListenFile)
                            .unwrap_or(false)
                    })
                })
            })
        };
        let previous = self.pointer.file_target;
        if previous != hit {
            if let Some(node) = previous {
                let pos = self.pointer_local(node, point);
                self.push(HostEvent::File(crate::contract::Routed {
                    window: self.window,
                    target: Some(crate::contract::RouteTarget { node, region: None }),
                    local: pos,
                    activation: None,
                    origin: self.frame.origin(),
                    input: kivact_types::FileDrop {
                        phase: FilePhase::Leave,
                        files: files.clone(),
                        position: point,
                    },
                }));
            }
            if let Some(node) = hit {
                let pos = self.pointer_local(node, point);
                self.push(HostEvent::File(crate::contract::Routed {
                    window: self.window,
                    target: Some(crate::contract::RouteTarget { node, region: None }),
                    local: pos,
                    activation: None,
                    origin: self.frame.origin(),
                    input: kivact_types::FileDrop {
                        phase: FilePhase::Enter,
                        files: files.clone(),
                        position: point,
                    },
                }));
            }
        }
        if let Some(node) = hit {
            let pos = self.pointer_local(node, point);
            let phase = if phase == kivact_types::FilePhase::Drop {
                FilePhase::Drop
            } else {
                FilePhase::Over
            };
            self.push(HostEvent::File(crate::contract::Routed {
                window: self.window,
                target: Some(crate::contract::RouteTarget { node, region: None }),
                local: pos,
                activation: None,
                origin: self.frame.origin(),
                input: kivact_types::FileDrop {
                    phase,
                    files,
                    position: point,
                },
            }));
        }
        self.pointer.file_target = if phase == kivact_types::FilePhase::Drop {
            None
        } else {
            hit
        };
    }
}
