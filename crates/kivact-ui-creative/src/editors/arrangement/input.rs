//! Role: Arrangement gesture transactions and discrete input.
use super::gesture::Snapshot;
use super::gesture::{Drag, LoopPart, State};
use super::layout::{automation_value, offset_y, pan_y, track_layout};
use crate::editors::arrangement::a11y::targets::Target;
use crate::editors::arrangement::model::{ArrangementRequest, TimelineTool};
use crate::editors::shared::metrics::ARRANGEMENT_SIDEBAR as LEFT;
use crate::editors::shared::model::ItemTime;
use crate::editors::shared::model::drawable;
use crate::editors::shared::region_parts::bounded_range;
use crate::editors::shared::selection;
use kivact::geometry::Point;
use kivact::interaction::DragApi as Gesture;
use kivact::view::{Handled, Key, KeyEvent, MouseButton, PointerEvent, PointerPhase, WheelEvent};
/// The press decides what the arrangement's drag does; the drag's sources
/// (the canvas and its regions) then follow the pointer until it ends.
pub(crate) fn pointer(state: State, e: PointerEvent) -> Handled {
    let PointerPhase::Down(press) = e.phase() else {
        return Handled::No;
    };
    if state.gesture.is_active() {
        return Handled::Yes;
    }
    match press.button {
        MouseButton::Middle => start(state, Drag::Pan, &e, None),
        MouseButton::Primary => {
            if let Some(region) = e.region() {
                let target = state.regions.with(|map| map.target(region.0));
                let (id, trim) = match target {
                    Some(Target::LoopStart) => return loop_drag(state, LoopPart::Start, &e),
                    Some(Target::LoopEnd) => return loop_drag(state, LoopPart::End, &e),
                    Some(Target::LoopBody) => return loop_drag(state, LoopPart::Body, &e),
                    Some(Target::LoopCreate) => return loop_drag(state, LoopPart::Create, &e),
                    // The press moves the playhead: see `drag`.
                    Some(Target::Playhead) => {
                        start(state, Drag::Playhead, &e, None);
                        return Handled::No;
                    }
                    Some(Target::AddTrack) => return Handled::No,

                    Some(Target::Item(id)) => (id, None),
                    Some(Target::Trim(id, right)) => (id, Some(right)),
                    Some(Target::Automation(track, id)) => {
                        state.active_auto.set(Some((track, id)));
                        start(state, Drag::Auto(track, id), &e, None);
                        return Handled::No;
                    }
                    _ => return Handled::No,
                };
                match state.tool.get() {
                    TimelineTool::Erase => {
                        state.edit.once(state.snapshot(&[id]), || {
                            state.request(ArrangementRequest::Erase(id))
                        });
                        return Handled::Yes;
                    }
                    TimelineTool::Mute => {
                        state.edit.once(state.snapshot(&[id]), || {
                            state.request(ArrangementRequest::ToggleClipMute(id))
                        });
                        return Handled::Yes;
                    }
                    TimelineTool::Split => {
                        let at = state.snapping(state.beat(e.pos().x));
                        state.edit.once(state.snapshot(&[id]), || {
                            state.request(ArrangementRequest::Split { clip: id, at })
                        });
                        return Handled::Yes;
                    }
                    TimelineTool::Select => {}
                }
                let mut pending = None;
                match selection::press(
                    &state.selection.get(),
                    id,
                    e.modifiers(),
                    &kivact::interaction::ListOptions::default(),
                ) {
                    selection::Press::Pending(id) => pending = Some(id),
                    selection::Press::Select(gesture) => state.select(id, gesture),
                }
                state.active_auto.set(None);
                start(
                    state,
                    if let Some(right) = trim {
                        Drag::Trim(id, right)
                    } else {
                        Drag::Clips
                    },
                    &e,
                    pending,
                );
            } else {
                let ts = state.track_data.get();
                for row in track_layout(&ts, offset_y(state)) {
                    if let Some((top, height)) = row.automation
                        && e.pos().y >= top
                        && e.pos().y < top + height
                        && e.pos().x >= LEFT
                    {
                        // The press adds the point: see `drag`.
                        start(state, Drag::AddPoint(row.id), &e, None);
                        state.active_auto.set(None);
                        return Handled::No;
                    }
                }
                start(state, Drag::Marquee(e.pos()), &e, None);
            }
        }
        _ => return Handled::No,
    }
    Handled::No
}

/// The arrangement's drag: a middle-button pan or the primary gesture the
/// press chose, one edit from its first change. The returned drag has the
/// callbacks its sources need; `state.gesture` is the same session.
pub(crate) fn drag(state: State) -> Gesture<Snapshot> {
    let drag = state
        .gesture
        .on_start(move |update| press_write(state, update.start_pos));
    state.press.install(
        drag,
        move || matches!(state.drag_state(), Some(Drag::Marquee(..))),
        (
            move || state.selection.get(),
            move |ids| state.selected.set(ids),
            move |id| state.select(id, kivact::interaction::ListGesture::Replace),
        ),
        (
            move |pos, origin, snapshot| motion(state, pos, origin, snapshot),
            move |rect| {
                // Clips use beats horizontally and lane pixels vertically.
                let left = state.beat(rect.x);
                let right = state.beat(rect.right());
                let rows = track_layout(&state.track_data.get(), offset_y(state));
                state
                    .data
                    .get()
                    .iter()
                    .filter(|clip| {
                        drawable(clip)
                            && rows
                                .iter()
                                .find(|row| row.id == clip.lane)
                                .is_some_and(|row| {
                                    // Keep application beat positions at f64 precision.
                                    clip.start <= right
                                        && clip.start + clip.duration >= left
                                        && row.top <= rect.bottom()
                                        && row.top + row.height >= rect.y
                                })
                    })
                    .map(|clip| clip.id)
                    .collect()
            },
            move |_| {},
            move |_| state.reset_fields(),
        ),
    )
}

/// The write of a press that changes a value by itself.
fn press_write(state: State, origin: Point) {
    match state.drag_state() {
        Some(Drag::Playhead) => seek(state, origin.x),
        Some(Drag::AddPoint(track)) => {
            let rows = track_layout(&state.track_data.get(), offset_y(state));
            if let Some((top, _)) = rows
                .iter()
                .find(|row| row.id == track)
                .and_then(|row| row.automation)
            {
                let beat = state
                    .snapping(state.beat(origin.x))
                    .clamp(0.0, state.length.get());
                let value = automation_value(origin.y, top);
                state
                    .gesture
                    .change(|| state.request(ArrangementRequest::AddPoint { track, beat, value }));
            }
        }
        _ => {}
    }
}

fn motion(state: State, pos: Point, origin: Point, snapshot: &Snapshot) {
    let scale = state.scale();
    let Some(action) = state.drag_state() else {
        return;
    };
    match action {
        Drag::Loop(mode) => {
            let Some(Snapshot::Loop((lo, hi))) = Some(snapshot.clone()) else {
                return;
            };
            let start = origin;
            let b = state.snapping(state.beat(pos.x));
            let delta = state.snapping(state.beat(pos.x) - state.beat(start.x));
            let unit = state
                .step()
                .clamp(state.min_duration.get().min(4.0), 4.0)
                .min(state.length.get());
            let range = match mode {
                LoopPart::Start => (b.min(hi - unit).max(0.0), hi),
                LoopPart::End => (lo, b.max(lo + unit).min(state.length.get())),
                LoopPart::Body => {
                    let a = (lo + delta).clamp(0.0, (state.length.get() - (hi - lo)).max(0.0));
                    (a, a + hi - lo)
                }
                LoopPart::Create => {
                    let a = state.snapping(state.beat(start.x));
                    (
                        a.min(b).min((state.length.get() - unit).max(0.0)),
                        a.max(b).max(a.min(b) + unit).min(state.length.get()),
                    )
                }
            };
            if range != state.range.get() {
                state.gesture.change(|| state.loop_range.set(range));
            }
        }
        Drag::Playhead => seek(state, pos.x),
        Drag::Clips => {
            let start = origin;
            if let Some(Snapshot::Clips(initial)) = Some(snapshot.clone()) {
                let dx = (pos.x - start.x) / scale;
                // The pressed clip snaps and the others keep their spacing to it.
                let primary = match state.press.pressed() {
                    Some(Target::Item(id)) => initial.iter().find(|v| v.0 == id),
                    _ => None,
                }
                .or(initial.first());
                let delta = primary.map_or(dx, |v| state.snapping(v.1 + dx) - v.1);
                let place = |start: f64, duration: f64| {
                    (start + delta).clamp(0.0, (state.length.get_untracked() - duration).max(0.0))
                };
                if delta != 0.0 || state.edit.is_active() {
                    state.gesture.change(|| {
                        state.request(ArrangementRequest::Retime(
                            initial
                                .iter()
                                .map(|v| ItemTime {
                                    id: v.0,
                                    time: place(v.1, v.2),
                                    duration: v.2,
                                })
                                .collect(),
                        ))
                    });
                }
            }
        }
        Drag::Trim(id, right) => {
            if let Some(Snapshot::Clips(initial)) = Some(snapshot.clone())
                && let Some(v) = initial.iter().find(|v| v.0 == id)
            {
                let dx = (pos.x - origin.x) / scale;
                let available = if right {
                    (state.length.get() - v.1).max(0.0)
                } else {
                    (v.1 + v.2).max(0.0).min(state.length.get())
                };
                let minimum = state.min_duration.get().min(available);
                let (time, duration) = if right {
                    (
                        v.1,
                        (v.2 + dx).clamp(minimum, (state.length.get() - v.1).max(minimum)),
                    )
                } else {
                    let dx = dx.clamp(-v.1, (v.2 - minimum).max(-v.1));
                    (v.1 + dx, v.2 - dx)
                };
                if dx != 0.0 || state.edit.is_active() {
                    state.gesture.change(|| {
                        state.request(ArrangementRequest::Retime(vec![ItemTime {
                            id,
                            time,
                            duration,
                        }]))
                    });
                }
            }
        }
        Drag::Auto(track, id) => move_automation(state, track, Some(id), pos, origin, snapshot),
        Drag::AddPoint(track) => move_automation(state, track, None, pos, origin, snapshot),
        Drag::Marquee(_) => {}
        Drag::Pan => {
            if let Some(Snapshot::View(_, p)) = Some(snapshot.clone()) {
                let from = origin;
                let next = Point::new(p.x + pos.x - from.x, pan_y(state, p.y + pos.y - from.y));
                if next != state.offset.get() {
                    state.gesture.change(|| state.pan.set(next));
                }
            }
        }
    }
}

pub(crate) fn key(state: State, e: KeyEvent) -> Handled {
    // The drag's sources take Escape during a drag.
    if e.key() == Key::Escape {
        state.choose.clear();
        return Handled::Yes;
    }
    if state.drag_state().is_some() {
        return Handled::Yes;
    }
    match e.key() {
        Key::Char('1') => state.tool_binding.set(TimelineTool::Select),
        Key::Char('3') => state.tool_binding.set(TimelineTool::Split),
        Key::Char('5') => state.tool_binding.set(TimelineTool::Erase),
        Key::Char('7') => state.tool_binding.set(TimelineTool::Mute),
        Key::Delete | Key::Backspace => {
            if let Some((i, id)) = state.active_auto.get_untracked() {
                if let Some(snapshot) = state.auto_snapshot(i, id) {
                    state.edit.once(snapshot, || {
                        state.request(ArrangementRequest::DeletePoint { track: i, id });
                    });
                }
            } else if !state.selection.get_untracked().is_empty() {
                let ids = state.selection.get_untracked();
                state.edit.once(state.snapshot(&ids), || {
                    state.request(ArrangementRequest::Delete(ids));
                });
                state.selected.set(Vec::new());
            }
        }
        _ if selection::select_all(&e) => state.choose.select_all(),
        Key::ArrowLeft | Key::ArrowRight if !state.selection.get_untracked().is_empty() => {
            let step = state.step() * if e.modifiers().shift { 10.0 } else { 1.0 };
            let dx = if e.key() == Key::ArrowLeft {
                -step
            } else {
                step
            };
            let snapshot = state.snapshot(&state.selection.get_untracked());
            let Snapshot::Clips(initial) = &snapshot else {
                return Handled::No;
            };
            let times = initial
                .iter()
                .map(|v| ItemTime {
                    id: v.0,
                    time: (v.1 + dx).clamp(0.0, (state.length.get_untracked() - v.2).max(0.0)),
                    duration: v.2,
                })
                .collect();
            state.edit.once(snapshot, || {
                state.request(ArrangementRequest::Retime(times))
            });
        }
        Key::Home => {
            state.edit.once(state.view_snapshot(), || {
                state.pan.set(Point::new(0.0, 0.0));
            });
        }
        _ => return Handled::No,
    }
    Handled::Yes
}

pub(crate) fn wheel(state: State, e: WheelEvent) -> Handled {
    if state.drag_state().is_some() {
        return Handled::Yes;
    }
    if !e.delta().x.is_finite() || !e.delta().y.is_finite() {
        return Handled::No;
    }
    let scale = state.scale();
    let offset = Point::new(state.offset.get().x, offset_y(state));
    let zooms = e.modifiers().ctrl || e.modifiers().meta;
    let (next_scale, next_offset) = if zooms {
        crate::editors::shared::geometry::wheel_zoom(
            scale,
            offset,
            e,
            (state.bounds.get().min(), state.bounds.get().max()),
            LEFT,
        )
    } else if e.modifiers().shift {
        (scale, Point::new(offset.x - e.delta().y, offset.y))
    } else {
        (
            scale,
            Point::new(offset.x, pan_y(state, offset.y - e.delta().y)),
        )
    };
    if (next_scale, next_offset) != (scale, offset) {
        state.edit.once(state.view_snapshot(), || {
            if zooms {
                state.zoom.set(next_scale);
            }
            state.pan.set(next_offset);
        });
    }
    Handled::Yes
}

fn loop_drag(state: State, part: LoopPart, e: &PointerEvent) -> Handled {
    start(state, Drag::Loop(part), e, None);
    Handled::No
}
pub(crate) fn source_snapshot(s: State) -> Snapshot {
    match s.drag_state() {
        Some(Drag::Clips) => s.snapshot(&s.selection.get_untracked()),
        Some(Drag::Trim(id, _)) => s.snapshot(&[id]),
        Some(Drag::Loop(_)) => Snapshot::Loop(bounded_range(
            s.range.get_untracked(),
            s.length.get_untracked(),
            s.loop_min_duration.get_untracked(),
        )),
        Some(Drag::Playhead) => Snapshot::Playhead(s.frame.get_untracked()),
        Some(Drag::Auto(track, id)) => s
            .auto_snapshot(track, id)
            .unwrap_or_else(|| s.snapshot(&[])),
        Some(Drag::AddPoint(track)) => Snapshot::Automation(track, 0, None),
        Some(Drag::Pan) => s.view_snapshot(),
        _ => s.snapshot(&[]),
    }
}
fn start(s: State, drag: Drag, e: &PointerEvent, pending: Option<u64>) {
    s.press.begin(
        e,
        e.region()
            .and_then(|id| s.regions.with(|regions| regions.target(id.0))),
        Some(drag),
        pending,
    );
}
fn seek(s: State, x: f64) {
    let value = s.beat(x);
    if value != s.frame.get() {
        s.gesture.change(|| s.playhead.set(value));
    }
}

fn move_automation(
    s: State,
    track: u64,
    id: Option<u64>,
    pos: Point,
    origin: Point,
    snapshot: &Snapshot,
) {
    let tracks = s.track_data.get();
    let rows = track_layout(&tracks, offset_y(s));
    let Some((top, _)) = rows
        .iter()
        .find(|row| row.id == track)
        .and_then(|row| row.automation)
    else {
        return;
    };
    let (beat, value) = if let Some(Snapshot::Automation(_, _, Some(initial))) =
        Some(snapshot.clone())
    {
        (
            s.snapping(initial.beat + (pos.x - origin.x) / s.scale())
                .clamp(0.0, s.length.get()),
            (initial.value - (pos.y - origin.y) / crate::editors::shared::metrics::AUTOMATION_SPAN)
                .clamp(0.0, 1.0),
        )
    } else {
        (
            s.snapping(s.beat(pos.x)).clamp(0.0, s.length.get()),
            automation_value(pos.y, top),
        )
    };
    if let Some(Snapshot::Automation(_, _, Some(initial))) = Some(snapshot.clone())
        && initial.beat == beat
        && initial.value == value
        && !s.edit.is_active()
    {
        return;
    }
    s.gesture.change(|| {
        s.request(if let Some(id) = id {
            ArrangementRequest::MovePoint {
                track,
                id,
                beat,
                value,
            }
        } else {
            ArrangementRequest::AddPoint { track, beat, value }
        })
    });
}
