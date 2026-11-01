//! Role: Arrangement semantic actions participate in the same edit transactions.
use super::super::gesture::Snapshot;
use super::super::gesture::State;
use crate::editors::arrangement::a11y::targets::Target;
use crate::editors::arrangement::model::ArrangementRequest;
use crate::editors::shared::region_parts::bounded_range;
use kivact::Callback;
use kivact::view::{A11yAction, A11yEvent};
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
    match event.action {
        A11yAction::Select => match target {
            Target::Item(id) => select(id, kivact::interaction::ListGesture::Replace),
            Target::Automation(track, id) => {
                active_auto.set(Some((track, id)));
            }
            _ => {}
        },
        A11yAction::Activate => match target {
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
        A11yAction::SetValue => {
            let Some(value) = event
                .value
                .and_then(|s| s.parse::<f64>().ok())
                .filter(|v| v.is_finite())
            else {
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
                    let (a, b) = bounded_range(range.get(), length, state.loop_min_duration);
                    let a = a.min(length);
                    let b = b.min(length);
                    let unit = state.loop_min_duration.min(length);
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
