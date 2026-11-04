//! Role: Pointer and discrete edits sharing one time-editor transaction.
use crate::editors::shared::geometry::wheel_zoom;
use crate::editors::shared::model::ItemTime;
use crate::editors::shared::selection;
use crate::editors::time::gesture::{Drag, Snapshot, State, Target};
use crate::editors::time::layout::item_rect;
use crate::editors::time::model::{TimeItem, TimeRequest};
use kivact::geometry::Point;
use kivact::interaction::{DragApi as Gesture, ListGesture};
use kivact::view::{Handled, Key, KeyEvent, MouseButton, PointerEvent, PointerPhase, WheelEvent};

/// The press decides what the editor's drag does; the drag's sources (the
/// canvas and its regions) then follow the pointer until it ends.
pub(crate) fn pointer<T: TimeItem>(s: State<T>, e: PointerEvent) -> Handled {
    let PointerPhase::Down(press) = e.phase() else {
        return Handled::No;
    };
    if s.gesture.is_active() {
        return Handled::Yes;
    }
    let target = s.target(e.region());
    match press.button {
        MouseButton::Middle => start(s, Drag::Pan, &e, None),
        MouseButton::Primary => {
            s.active.set(target);
            match target {
                // A lane press only activates on a stationary double click.
                Some(Target::Lane(_)) => s.press.begin(&e, target, None, None),
                // The press moves the playhead: see `drag`.
                Some(Target::Playhead) => start(s, Drag::Playhead, &e, None),
                Some(Target::Trim(id, right)) => {
                    s.choose.select(id, ListGesture::Replace);
                    start(s, Drag::Trim(id, right), &e, None);
                }
                Some(Target::Item(id)) => {
                    let mut pending = None;
                    match selection::press(
                        &s.selection.get(),
                        id,
                        e.modifiers(),
                        &kivact::interaction::ListOptions::default(),
                    ) {
                        selection::Press::Pending(id) => pending = Some(id),
                        selection::Press::Select(gesture) => {
                            s.choose.select(id, gesture);
                        }
                    }
                    start(s, Drag::Items, &e, pending);
                }
                None => start(s, Drag::Marquee(e.pos(), e.pos()), &e, None),
            }
        }
        _ => return Handled::No,
    }
    Handled::No
}

/// The editor's drag: a middle-button pan or the primary gesture the press
/// chose, one edit from its first change. The returned drag has the callbacks
/// its sources need; `s.gesture` is the same session.
pub(crate) fn drag<T: TimeItem>(s: State<T>) -> Gesture<Snapshot> {
    let drag = s.gesture.on_start(move |update| {
        if s.drag_state() == Some(Drag::Playhead) {
            motion(s, update.pos, update.start_pos, &update.payload);
        }
    });
    s.press.install(
        drag,
        move || matches!(s.drag_state(), Some(Drag::Marquee(..))),
        (
            move || s.selection.get(),
            move |ids| s.selected.set(ids),
            move |id| {
                s.choose.select(id, ListGesture::Replace);
            },
        ),
        (
            move |pos, origin, snapshot| motion(s, pos, origin, snapshot),
            move |region| {
                s.data
                    .get()
                    .iter()
                    .filter(|item| {
                        item_rect(
                            *item,
                            &s.visible.get(),
                            s.lane_height,
                            s.scale
                                .get()
                                .clamp(s.bounds.get().min(), s.bounds.get().max()),
                            s.offset.get(),
                        )
                        .is_some_and(|r| r.intersects(region))
                    })
                    .map(TimeItem::id)
                    .collect()
            },
            move |target| s.activate(target),
            move |_| s.reset_fields(),
        ),
    )
}
fn start<T: TimeItem>(s: State<T>, drag: Drag, e: &PointerEvent, pending: Option<u64>) {
    s.press.begin(e, s.target(e.region()), Some(drag), pending);
}
fn motion<T: TimeItem>(s: State<T>, pos: Point, origin: Point, snapshot: &Snapshot) {
    let Some(drag) = s.drag_state() else {
        return;
    };
    let snapshot = snapshot.clone();
    if matches!(drag, Drag::Items | Drag::Trim(..)) && !s.press.moved() {
        return;
    }
    let snap = |value: f64| {
        if s.snap.get_untracked() {
            let step = s.step();
            let rounded = (value / step).round() * step;
            if rounded.is_finite() { rounded } else { value }
        } else {
            value
        }
    };
    match (drag, snapshot) {
        (Drag::Playhead, _) => {
            let value = snap(s.time(pos.x)).max(0.0);
            if value != s.playhead.get() {
                s.gesture.change(|| s.playhead.set(value));
            }
        }
        (Drag::Marquee(from, _), _) => s.set_drag(Some(Drag::Marquee(from, pos))),
        (Drag::Pan, Snapshot::View(_, p)) => {
            let next = Point::new(p.x + pos.x - origin.x, p.y + pos.y - origin.y);
            if next != s.offset.get() {
                s.gesture.change(|| s.pan.set(next));
            }
        }
        (Drag::Items, Snapshot::Items(initial)) => {
            let dx = (pos.x - origin.x)
                / s.scale
                    .get()
                    .clamp(s.bounds.get().min(), s.bounds.get().max());
            // The pressed item snaps and the others keep their spacing to it.
            let primary = match s.press.pressed() {
                Some(Target::Item(id)) => initial.iter().find(|v| v.0 == id),
                _ => None,
            }
            .or(initial.first());
            let (delta, snapped) = match primary {
                Some(v) if s.snap.get() => {
                    let time = snap(v.1 + dx);
                    (time - v.1, Some((v.0, time)))
                }
                _ => (dx, None),
            };
            let placed = |v: &(u64, f64, f64, u64)| match snapped {
                Some((id, time)) if id == v.0 => time,
                _ => v.1 + delta,
            };
            if initial.iter().any(|v| v.1 != placed(v)) || s.edit.is_active() {
                s.gesture.change(|| {
                    s.request(TimeRequest::Retime(
                        initial
                            .iter()
                            .map(|v| ItemTime {
                                id: v.0,
                                time: placed(v),
                                duration: v.2,
                            })
                            .collect(),
                    ))
                });
            }
        }
        (Drag::Trim(id, right), Snapshot::Items(initial)) => {
            if let Some(v) = initial.iter().find(|v| v.0 == id) {
                let dx = (pos.x - origin.x)
                    / s.scale
                        .get()
                        .clamp(s.bounds.get().min(), s.bounds.get().max());
                let (time, duration) = if right {
                    (v.1, (snap(v.1 + v.2 + dx) - v.1).max(s.min_duration.get()))
                } else {
                    let time = snap(v.1 + dx).min(v.1 + v.2 - s.min_duration.get());
                    (time, v.1 + v.2 - time)
                };
                if time != v.1 || duration != v.2 || s.edit.is_active() {
                    s.gesture.change(|| {
                        s.request(TimeRequest::Retime(vec![ItemTime { id, time, duration }]))
                    });
                }
            }
        }
        _ => {}
    }
}
pub(crate) fn wheel<T: TimeItem>(s: State<T>, e: WheelEvent) -> Handled {
    if s.drag_state().is_some() {
        return Handled::Yes;
    }
    if !e.delta().x.is_finite() || !e.delta().y.is_finite() {
        return Handled::No;
    }
    let scale = s
        .scale
        .get()
        .clamp(s.bounds.get().min(), s.bounds.get().max());
    let offset = s.offset.get();
    let zooms = e.modifiers().ctrl || e.modifiers().meta;
    let (next_scale, next_offset) = if zooms {
        wheel_zoom(
            scale,
            offset,
            e,
            (s.bounds.get().min(), s.bounds.get().max()),
            crate::editors::shared::metrics::SIDEBAR,
        )
    } else {
        (scale, Point::new(offset.x - e.delta().y, offset.y))
    };
    if (next_scale, next_offset) != (scale, offset) {
        s.edit.once(s.view_snapshot(), || {
            if zooms {
                s.zoom.set(next_scale);
            }
            s.pan.set(next_offset);
        });
    }
    Handled::Yes
}
pub(crate) fn key<T: TimeItem>(s: State<T>, e: KeyEvent) -> Handled {
    if s.child_focus.get_untracked() {
        return Handled::No;
    }
    // The drag's sources take Escape during a drag.
    if e.key() == Key::Escape {
        s.choose.clear();
        return Handled::Yes;
    }
    if s.drag_state().is_some() {
        return Handled::Yes;
    }
    let step = s.step() * if e.modifiers().shift { 10.0 } else { 1.0 };
    match e.key() {
        Key::Enter => {
            if !e.repeat()
                && let Some(target) = s.target(e.region()).or(s.active.get_untracked())
            {
                s.activate(target);
            }
        }
        Key::Home => {
            s.edit.once(s.view_snapshot(), || {
                s.pan.set(Point::new(0.0, 0.0));
                s.zoom
                    .set(1.0_f64.clamp(s.bounds.get().min(), s.bounds.get().max()));
            });
        }
        _ if selection::select_all(&e) => s.choose.select_all(),
        Key::Delete | Key::Backspace => {
            let ids = s.selected_ids();
            if !ids.is_empty() {
                s.edit.once(s.snapshot(&ids), || {
                    s.request(TimeRequest::Delete(ids.clone()));
                });
                s.choose.clear();
            }
        }
        Key::ArrowLeft | Key::ArrowRight => {
            let dx = if e.key() == Key::ArrowLeft {
                -step
            } else {
                step
            };
            if !dx.is_finite() {
                return Handled::Yes;
            }
            let playhead = e.region().is_some_and(|id| {
                s.scene.with_untracked(|v| {
                    v.as_ref().ok().and_then(|v| v.target(id)).copied() == Some(Target::Playhead)
                })
            });
            if playhead {
                s.set_playhead((s.playhead.get_untracked() + dx).max(0.0));
            } else {
                let ids = s.selected_ids();
                if s.data.with_untracked(|items| {
                    items
                        .iter()
                        .any(|item| ids.contains(&item.id()) && !(item.time() + dx).is_finite())
                }) {
                    return Handled::Yes;
                }
                if !ids.is_empty() {
                    let proposals = s
                        .data
                        .get_untracked()
                        .iter()
                        .filter(|i| ids.contains(&i.id()))
                        .map(|i| ItemTime {
                            id: i.id(),
                            time: i.time() + dx,
                            duration: i.duration(),
                        })
                        .collect();
                    s.edit.once(s.snapshot(&ids), || {
                        s.request(TimeRequest::Retime(proposals));
                    });
                }
            }
        }
        _ => return Handled::No,
    }
    Handled::Yes
}

impl<T: TimeItem> State<T> {
    fn target(self, region: Option<kivact::draw::RegionId>) -> Option<Target> {
        region.and_then(|id| {
            self.scene.with_untracked(|scene| {
                scene
                    .as_ref()
                    .ok()
                    .and_then(|scene| scene.target(id).copied())
            })
        })
    }
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
            Target::Item(id) | Target::Trim(id, _) => {
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
            Some(Target::Item(id) | Target::Trim(id, _)) => {
                if let Some(item) = self
                    .data
                    .with_untracked(|items| items.iter().find(|item| item.id() == id).cloned())
                {
                    return EditorContext {
                        item: Some(id),
                        lane: Some(item.lane()),
                        time: item.time(),
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
    pub(crate) fn context_request(self, region: Option<kivact::draw::RegionId>) {
        if self.drag_state().is_some() {
            return;
        }
        let selected = self.selection.get_untracked();
        let lanes = self.visible.get_untracked();
        let fallback = self.data.with_untracked(|items| {
            selected.iter().find_map(|id| {
                items
                    .iter()
                    .find(|item| {
                        item.id() == *id && lanes.iter().any(|lane| lane.id == item.lane())
                    })
                    .map(|item| Target::Item(item.id()))
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
    pub(crate) fn context_key(self, event: &KeyEvent) -> Handled {
        crate::editors::shared::context::context_key(
            event,
            self.child_focus.get(),
            self.drag_state().is_some(),
            |region| self.context_request(region),
        )
    }
    pub(crate) fn context_pointer(self, event: PointerEvent) -> Handled {
        crate::editors::shared::context::context_pointer(
            event,
            self.drag_state().is_some(),
            self.target(event.region()),
            self.context,
            |target| self.context_for(target),
            |x| self.time(x).max(0.0),
        )
    }
}
