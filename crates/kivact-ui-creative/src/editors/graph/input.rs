//! Role: Graph pointer and key gestures report requests and edit bound view values.

use crate::editors::graph::gesture::{DragKind, Gesture, Snapshot, State, Target};
use crate::editors::graph::model::Interpolation;

use crate::editors::shared::selection;
use kivact::draw::Point;
use kivact::interaction::{Drag, ListGesture};
use kivact::view::{Handled, Key, KeyEvent, MouseButton, PointerEvent, PointerPhase, WheelEvent};

impl State {
    /// The press decides what the editor's drag does; the drag's sources (the
    /// canvas and its regions) then follow the pointer until it ends.
    pub(crate) fn pointer(self, event: PointerEvent) -> Handled {
        match event.phase() {
            PointerPhase::Down(press) => {
                if self.gesture.is_active() {
                    return Handled::Yes;
                }
                let pan = press.button == MouseButton::Middle
                    || (press.button == MouseButton::Primary && self.space.armed().get_untracked());
                if !pan && press.button != MouseButton::Primary {
                    return Handled::No;
                }
                let target = self.target(event.region());
                let mut single = None;
                let (kind, initial) = if pan {
                    (DragKind::Pan, Some(Snapshot::View(self.view.get())))
                } else {
                    match target {
                        Some(Target::Key(id)) => {
                            self.active.set(Some(Target::Key(id)));
                            match selection::press(
                                &self.selected.get(),
                                id,
                                event.modifiers(),
                                &kivact::interaction::ListPolicy::default(),
                            ) {
                                selection::Press::Pending(id) => single = Some(id),
                                selection::Press::Select(gesture) => {
                                    self.choose.select(id, gesture);
                                }
                            }
                            let ids = self.ids();
                            let initial = self.positions(&ids);
                            (DragKind::Keys(ids), Some(initial))
                        }
                        Some(Target::Tangent(id, incoming)) => {
                            self.select(Target::Tangent(id, incoming), true);
                            (
                                DragKind::Tangent(id, incoming),
                                Some(self.tangent_snapshot(id, incoming)),
                            )
                        }
                        Some(Target::Lane(id)) => {
                            self.active.set(Some(Target::Lane(id)));
                            (DragKind::Lane, None)
                        }
                        // The press moves the playhead: see `drag`.
                        Some(Target::Playhead) => {
                            self.active.set(Some(Target::Playhead));
                            (
                                DragKind::Playhead,
                                Some(Snapshot::Playhead(self.playhead.get())),
                            )
                        }
                        _ => {
                            self.active.set(None);
                            (DragKind::Marquee(event.pos(), event.pos()), None)
                        }
                    }
                };
                self.press
                    .begin(&event, target, (Some(Gesture { kind }), initial), single);
                Handled::No
            }
            _ => Handled::No,
        }
    }
    /// The editor's drag: a middle-button or Space pan, or the primary
    /// gesture the press chose, one edit from its first change. The returned
    /// drag has the callbacks its sources need; `self.gesture` is the same
    /// session.
    pub(crate) fn drag(self) -> Drag<()> {
        let drag = self.gesture.on_start(move |_| {
            if self
                .drag_state()
                .is_some_and(|gesture| gesture.kind == DragKind::Playhead)
            {
                let time = self.geometry().time(self.press.origin().x);
                self.gesture.change(|| self.playhead.set(time));
            }
        });
        self.press.install(
            drag,
            move || {
                self.drag_state()
                    .is_some_and(|gesture| matches!(gesture.kind, DragKind::Marquee(..)))
            },
            (
                move || self.selected.get(),
                move |ids| self.selected.set(ids),
                move |id| {
                    self.choose.select(id, ListGesture::Replace);
                },
            ),
            (
                move |pos| self.motion(pos),
                move |rect| {
                    let space = self.geometry();
                    self.visible.with(|keys| {
                        keys.iter()
                            .filter(|key| space.key(key).intersects(rect))
                            .map(|key| key.key.id)
                            .collect()
                    })
                },
                move |target| self.activate(target),
                move |_| self.reset_fields(),
            ),
        )
    }
    fn motion(self, point: Point) {
        let Some(mut gesture) = self.drag_state() else {
            return;
        };
        let origin = self.press.origin();
        if point == origin && !self.press.moved() {
            return;
        }
        let initial = self.initial();
        let space = self.geometry();
        let total = Point::new(point.x - origin.x, point.y - origin.y);
        match &mut gesture.kind {
            DragKind::Keys(_) | DragKind::Tangent(_, _) => {
                if let Some(initial) = &initial {
                    let total = space.delta(total);
                    self.gesture
                        .change(|| self.drag_fields(initial, total.0, total.1));
                }
            }
            DragKind::Playhead => self
                .gesture
                .change(|| self.playhead.set(space.time(point.x))),
            DragKind::Pan => {
                if let Some(Snapshot::View(initial)) = initial {
                    let next = crate::editors::graph::layout::Space::new(initial, self.size.get())
                        .pan(total);
                    self.gesture.change(|| self.view.set(next));
                }
            }
            DragKind::Marquee(_, to) => *to = point,
            DragKind::Lane => {}
        }
        self.set_drag(Some(gesture));
    }
    pub(crate) fn key(self, event: KeyEvent) -> Handled {
        if self.child_focus.get_untracked() {
            return Handled::No;
        }
        // The drag's sources take Escape during a drag.
        if self.drag_state().is_some() {
            return Handled::Yes;
        }
        let target = self
            .target(event.region())
            .or(self.active.get_untracked())
            .filter(|target| {
                self.scene.with_untracked(|scene| {
                    scene.as_ref().ok().is_some_and(|scene| scene.live(target))
                })
            });
        match event.key() {
            Key::Enter => {
                if !event.repeat()
                    && let Some(target) = target
                {
                    self.activate(target);
                }
            }
            Key::Escape => {
                self.choose.clear();
                self.active.set(None);
            }
            _ if selection::select_all(&event) => self.choose.select_all(),
            Key::Delete | Key::Backspace => self.delete(),
            Key::Char('1') => self.interpolate(Interpolation::Constant),
            Key::Char('2') => self.interpolate(Interpolation::Linear),
            Key::Char('3') => self.interpolate(Interpolation::Bezier),
            Key::Char('f' | 'F') => self.fit(false),
            Key::Home => self.fit(true),
            Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown => {
                let step = self.step();
                let (time, value) = match event.key() {
                    Key::ArrowLeft => (-step.0, 0.0),
                    Key::ArrowRight => (step.0, 0.0),
                    Key::ArrowUp => (0.0, step.1),
                    _ => (0.0, -step.1),
                };
                match target {
                    Some(Target::Playhead) => self.change(
                        Target::Playhead,
                        None,
                        if time != 0.0 {
                            time
                        } else {
                            value.signum() * step.0
                        },
                    ),
                    Some(Target::Tangent(id, incoming)) => {
                        let initial = self.tangent_snapshot(id, incoming);
                        self.edit.once(initial.clone(), || {
                            self.propose(&initial, time, value, self.snap.get_untracked());
                        });
                    }
                    _ => {
                        let ids = self.ids();
                        if ids.is_empty() {
                            return Handled::Yes;
                        }
                        let initial = self.positions(&ids);
                        self.edit.once(initial.clone(), || {
                            self.propose(&initial, time, value, self.snap.get_untracked());
                        });
                    }
                }
            }
            _ => return Handled::No,
        }
        Handled::Yes
    }
    pub(crate) fn wheel(self, event: WheelEvent) -> Handled {
        if self.drag_state().is_some() {
            return Handled::Yes;
        }
        if !(event.modifiers().ctrl || event.modifiers().meta)
            || !event.delta().x.is_finite()
            || !event.delta().y.is_finite()
        {
            return Handled::No;
        }
        let next = self
            .geometry()
            .zoom(event.pos(), event.delta().y, event.modifiers().shift);
        if next != self.view.get() {
            self.set_view(next);
        }
        Handled::Yes
    }
}

impl State {
    pub(crate) fn activate(self, target: Target) {
        if self.drag_state().is_some()
            || !self.scene.with_untracked(|scene| {
                scene.as_ref().ok().is_some_and(|scene| scene.live(&target))
            })
        {
            return;
        }
        match target {
            Target::Lane(id) => {
                if let Some(callback) = self.lane_activate {
                    callback.call(id);
                }
            }
            Target::Key(id) | Target::Tangent(id, _) => {
                if let Some(callback) = self.item_activate {
                    callback.call(id);
                }
            }
            Target::Playhead => {}
        }
    }
    fn context_for(self, target: Option<Target>) -> crate::editors::shared::model::EditorContext {
        use crate::editors::shared::model::EditorContext;
        match target {
            Some(Target::Key(id) | Target::Tangent(id, _)) => {
                if let Some(key) = self
                    .visible
                    .with_untracked(|keys| keys.iter().find(|key| key.key.id == id).cloned())
                {
                    return EditorContext {
                        item: Some(id),
                        lane: Some(key.curve),
                        time: key.key.time,
                    };
                }
            }
            Some(Target::Lane(id)) => {
                return EditorContext {
                    item: None,
                    lane: Some(id),
                    time: self.playhead.get_untracked(),
                };
            }
            _ => {}
        }
        EditorContext {
            item: None,
            lane: None,
            time: self.playhead.get_untracked(),
        }
    }
    pub(crate) fn context_pointer(self, event: PointerEvent) -> Handled {
        crate::editors::shared::context::context_pointer(
            event,
            self.drag_state().is_some(),
            self.target(event.region()),
            self.context,
            |target| self.context_for(target),
            |x| self.geometry().time(x),
        )
    }
    pub(crate) fn context_key(self, event: &KeyEvent) -> Handled {
        crate::editors::shared::context::context_key(
            event,
            self.child_focus.get(),
            self.drag_state().is_some(),
            |region| self.context_request(region),
        )
    }
}

impl State {
    pub(crate) fn context_request(self, region: Option<kivact::draw::RegionId>) {
        let selected = self.selected.get_untracked();
        let fallback = self.visible.with_untracked(|keys| {
            selected.iter().find_map(|id| {
                keys.iter()
                    .find(|key| key.key.id == *id)
                    .map(|key| Target::Key(key.key.id))
            })
        });
        self.scene.with_untracked(|scene| {
            crate::editors::shared::context::context_request(
                scene.as_ref().ok().map(|scene| scene.as_ref()),
                region,
                self.active.get_untracked(),
                fallback,
                self.context,
                |target| self.context_for(target),
            );
        });
    }
}
