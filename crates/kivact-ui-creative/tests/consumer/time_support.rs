//! Role: Consumer-owned clip and keyframe edits for the timeline and dope sheet.

use kivact::prelude::*;
use kivact::state::callback;
use kivact_ui_creative as ui;

pub(crate) fn apply_clips(items: &mut Vec<ui::Clip>, request: &ui::TimeRequest) {
    match request {
        ui::TimeRequest::Retime(values) => {
            for item in items {
                if let Some(value) = values.iter().find(|value| value.id == item.id) {
                    item.start = value.time;
                    item.duration = value.duration;
                }
            }
        }
        ui::TimeRequest::Delete(ids) => items.retain(|item| !ids.contains(&item.id)),
    }
}

pub(crate) fn apply_keys(items: &mut Vec<ui::Keyframe>, request: &ui::TimeRequest) {
    match request {
        ui::TimeRequest::Retime(values) => {
            for item in items {
                if let Some(value) = values.iter().find(|value| value.id == item.id) {
                    item.time = value.time;
                    item.duration = value.duration;
                }
            }
        }
        ui::TimeRequest::Delete(ids) => items.retain(|item| !ids.contains(&item.id)),
    }
}

pub(crate) trait TestTime: kivact::state::PropValue {
    fn timing(&self) -> ui::ItemTime;
}

impl TestTime for ui::Clip {
    fn timing(&self) -> ui::ItemTime {
        ui::ItemTime {
            id: self.id,
            time: self.start,
            duration: self.duration,
        }
    }
}

impl TestTime for ui::Keyframe {
    fn timing(&self) -> ui::ItemTime {
        ui::ItemTime {
            id: self.id,
            time: self.time,
            duration: self.duration,
        }
    }
}

pub(crate) fn time_edits<T: TestTime, R: kivact::view::HandlerResult>(
    items: impl kivact::state::IntoBinding<Vec<T>>,
    apply: fn(&mut Vec<T>, &ui::TimeRequest),
    observer: impl Fn(kivact::view::EditPhase) -> R + 'static,
) -> impl Fn(ui::EditorEvent<ui::TimeRequest>) + 'static {
    let items = items.into_binding();
    let initial = signal(None::<Vec<T>>);
    let observer = callback(observer);
    move |event| {
        use kivact::view::EditPhase;
        match event {
            ui::EditorEvent::Begin => {
                initial.set(Some(items.get_untracked()));
                observer.call(EditPhase::Begin);
            }
            ui::EditorEvent::Request(request) => {
                let mut next = items.get_untracked();
                apply(&mut next, &request);
                items.set(next);
            }
            ui::EditorEvent::Commit => {
                initial.set(None);
                observer.call(EditPhase::Commit);
            }
            ui::EditorEvent::Cancel => {
                if let Some(value) = initial.get_untracked() {
                    let mut next = items.get_untracked();
                    apply(
                        &mut next,
                        &ui::TimeRequest::Retime(value.iter().map(TestTime::timing).collect()),
                    );
                    items.set(next);
                }
                initial.set(None);
                observer.call(EditPhase::Cancel);
            }
        }
    }
}

pub(crate) fn app_timeline(
    items: impl kivact::state::IntoBinding<Vec<ui::Clip>>,
    state: ui::TimeState,
) -> ui::Timeline {
    let items = items.into_binding();
    ui::timeline(items, state).on_edit(time_edits(items, apply_clips, |_| {}))
}

pub(crate) fn app_dope_sheet(
    items: impl kivact::state::IntoBinding<Vec<ui::Keyframe>>,
    state: ui::TimeState,
) -> ui::DopeSheet {
    let items = items.into_binding();
    ui::dope_sheet(items, state).on_edit(time_edits(items, apply_keys, |_| {}))
}
