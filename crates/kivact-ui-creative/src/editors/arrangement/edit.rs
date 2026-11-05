//! Role: Arrangement edits, their restoration and the accessible actions sharing them.

use super::gesture::{Snapshot, State};
use crate::editors::arrangement::a11y::targets::Target;
use crate::editors::arrangement::model::ArrangementRequest;
use crate::editors::shared::region_parts::bounded_range;
use kivact::geometry::Point;
use kivact::state::Callback;
use kivact::view::{A11yEvent, A11yRequest};

/// Restores only bound scalar and view values; the application restores items.
pub(crate) fn restore(s: Restore, snapshot: Snapshot) {
    match snapshot {
        Snapshot::Clips(..) | Snapshot::Automation(..) => {}
        Snapshot::Playhead(v) => s.playhead.set(v),
        Snapshot::Loop(v) => s.loop_range.set(v),
        Snapshot::Cycle(v) => s.cycling.set(v),
        Snapshot::View(pose) => s.transform.set(pose),
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Restore {
    pub playhead: kivact::state::Binding<f64>,
    pub loop_range: kivact::state::Binding<(f64, f64)>,
    pub cycling: kivact::state::Binding<bool>,
    pub transform: kivact::state::Binding<kivact::interaction::ViewTransform>,
}

impl State {
    pub(crate) fn set_scale(self, scale: f64) {
        self.transform.set(kivact::interaction::ViewTransform {
            scale,
            ..self.transform.get_untracked()
        });
    }
    pub(crate) fn set_offset(self, offset: Point) {
        self.transform.set(kivact::interaction::ViewTransform {
            offset,
            ..self.transform.get_untracked()
        });
    }

    pub(crate) fn request(self, request: crate::editors::arrangement::model::ArrangementRequest) {
        if let Some(callback) = self.on_edit {
            callback.call(crate::editors::shared::model::EditorEvent::Request(request));
        }
    }

    pub(crate) fn snapshot(self, ids: &[u64]) -> Snapshot {
        Snapshot::Clips(
            self.data
                .get_untracked()
                .iter()
                .filter(|c| ids.contains(&c.id))
                .map(|c| (c.id, c.start, c.duration))
                .collect(),
        )
    }
    pub(crate) fn view_snapshot(self) -> Snapshot {
        Snapshot::View(self.transform.get_untracked())
    }
    pub(crate) fn auto_snapshot(self, track: u64, id: u64) -> Option<Snapshot> {
        self.track_data.with_untracked(|tracks| {
            tracks.iter().find(|t| t.id == track).map(|t| {
                Snapshot::Automation(t.id, id, t.automation.iter().find(|p| p.id == id).cloned())
            })
        })
    }
    /// Stop the drag without a callback, canceling its edit.
    pub(crate) fn cancel(self) {
        self.gesture.cancel();
        self.reset();
    }
    /// Forget the gesture of a drag that ended.
    pub(crate) fn reset(self) {
        self.press.cancel(|| self.reset_fields());
    }
    pub(crate) fn reset_fields(self) {}
}

pub(crate) fn a11y(state: State, add_track: Option<Callback<()>>, event: A11yEvent) {
    if state.drag_state().is_some() {
        return;
    }
    let State {
        track_data,
        regions,
        active_auto,
        selection,
        cycling,
        cycle,
        playhead,
        frame,
        range,
        loop_range,
        ..
    } = state;
    let select = move |id, gesture| state.select(id, gesture);
    let Some(target) = event
        .region
        .and_then(|r| regions.with(|map| map.target(r.0)))
    else {
        return;
    };
    match event.request {
        A11yRequest::Select => match target {
            Target::Item(id) => select(id, kivact::interaction::ListGesture::Replace),
            Target::Automation(track, id) => {
                active_auto.set(Some((track, id)));
            }
            _ => {}
        },
        A11yRequest::Activate => match target {
            Target::AddTrack => {
                if let Some(callback) = add_track {
                    callback.call(());
                }
            }
            Target::Track(id, tag) => {
                if let Some(old) =
                    track_data.with(|v| v.iter().find(|t| t.id == id).map(|t| tag.checked(t)))
                {
                    state.edit.once(state.snapshot(&[]), || {
                        state.request(ArrangementRequest::TrackFlag {
                            track: id,
                            flag: tag,
                            value: !old,
                        });
                    });
                }
            }
            Target::LoopBody => {
                state.edit.once(Snapshot::Cycle(cycle.get()), || {
                    cycling.set(!cycle.get());
                });
            }
            Target::Item(id) => {
                if !selection.get().contains(&id) {
                    select(id, kivact::interaction::ListGesture::Replace);
                }
            }
            Target::Trim(id, _) => select(id, kivact::interaction::ListGesture::Replace),
            _ => {}
        },
        A11yRequest::SetValue(text) => {
            let Some(value) = text.parse::<f64>().ok().filter(|v| v.is_finite()) else {
                return;
            };
            match target {
                Target::Playhead => {
                    state.edit.once(Snapshot::Playhead(frame.get()), || {
                        playhead.set(value.clamp(0.0, state.length.get()));
                    });
                }
                Target::LoopStart | Target::LoopEnd => {
                    let length = state.length.get();
                    let (a, b) = bounded_range(range.get(), length, state.loop_min_duration.get());
                    let a = a.min(length);
                    let b = b.min(length);
                    let unit = state.loop_min_duration.get().min(length);
                    state.edit.once(Snapshot::Loop(range.get()), || {
                        loop_range.set(if target == Target::LoopStart {
                            (value.clamp(0.0, (b - unit).max(0.0)), b)
                        } else {
                            (a, value.clamp((a + unit).min(length), length))
                        });
                    });
                }
                Target::Automation(track, id) => {
                    if let Some(snapshot) = state.auto_snapshot(track, id) {
                        state.edit.once(snapshot.clone(), || {
                            if let Snapshot::Automation(_, _, Some(point)) = snapshot.clone() {
                                state.request(ArrangementRequest::MovePoint {
                                    track,
                                    id,
                                    beat: point.beat,
                                    value: value.clamp(0.0, 1.0),
                                });
                            }
                        });
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}
