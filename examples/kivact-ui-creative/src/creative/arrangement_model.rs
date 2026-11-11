//! Role: Application reducer and gesture snapshots for arrangement requests.
use super::txn::Txn;
use kivact::state::Signal;
use kivact_ui_creative as ui;

/// The application's arrangement items and the provisional automation point
/// one drag creates, kept for the length of the edit.
#[derive(Clone, PartialEq)]
pub(super) struct ArrangementEdit {
    clips: Vec<ui::Clip>,
    tracks: Vec<ui::TimelineTrack>,
    provisional: Option<u64>,
}

pub(super) fn edits(
    clips: Signal<Vec<ui::Clip>>,
    tracks: Signal<Vec<ui::TimelineTrack>>,
    observe: impl Fn(kivact::view::EditPhase) + 'static,
) -> impl Fn(ui::EditorEvent<ui::ArrangementRequest>) + 'static {
    let txn = Txn::signal();
    move |event| {
        use kivact::view::EditPhase;
        match event {
            ui::EditorEvent::Begin => {
                Txn::begin(
                    txn,
                    ArrangementEdit {
                        clips: clips.get_untracked(),
                        tracks: tracks.get_untracked(),
                        provisional: None,
                    },
                );
                observe(EditPhase::Begin);
            }
            ui::EditorEvent::Request(request) => {
                clips.update(|clips| tracks.update(|tracks| apply(clips, tracks, &request, txn)));
            }
            ui::EditorEvent::Commit => {
                Txn::commit(txn);
                observe(EditPhase::Commit);
            }
            ui::EditorEvent::Cancel => {
                if let Some(edit) = Txn::cancel(txn) {
                    clips.set(edit.clips);
                    tracks.set(edit.tracks);
                }
                observe(EditPhase::Cancel);
            }
        }
    }
}
/// The provisional automation point of the running edit, created once.
fn provisional_id(txn: Signal<Txn<ArrangementEdit>>, tracks: &[ui::TimelineTrack]) -> u64 {
    Txn::read(txn, |edit| edit.provisional)
        .flatten()
        .unwrap_or_else(|| {
            let id = next_id(tracks.iter().flat_map(|t| &t.automation).map(|p| p.id));
            Txn::update_snapshot(txn, |edit| edit.provisional = Some(id));
            id
        })
}

fn next_id(ids: impl Iterator<Item = u64>) -> u64 {
    let used = ids.collect::<std::collections::HashSet<_>>();
    let mut id = used.iter().copied().max().unwrap_or(0).wrapping_add(1);
    while used.contains(&id) {
        id = id.wrapping_add(1);
    }
    id
}
fn apply(
    clips: &mut Vec<ui::Clip>,
    tracks: &mut [ui::TimelineTrack],
    request: &ui::ArrangementRequest,
    txn: Signal<Txn<ArrangementEdit>>,
) {
    use ui::ArrangementRequest as R;
    match request {
        R::Retime(times) => {
            for time in times {
                if let Some(clip) = clips.iter_mut().find(|c| c.id == time.id) {
                    clip.start = time.time;
                    clip.duration = time.duration;
                }
            }
        }
        R::Delete(ids) => clips.retain(|c| !ids.contains(&c.id)),
        R::Erase(id) => clips.retain(|c| c.id != *id),
        R::ToggleClipMute(id) => {
            if let Some(c) = clips.iter_mut().find(|c| c.id == *id) {
                c.muted = !c.muted;
            }
        }
        R::Split { clip, at } => {
            let id = next_id(clips.iter().map(|c| c.id));
            if let Some(c) = clips.iter_mut().find(|c| c.id == *clip)
                && *at > c.start
                && *at < c.start + c.duration
            {
                let mut right = c.clone();
                right.id = id;
                right.label = format!("{} (2)", c.label);
                // A media application also updates its own source-offset model here.
                right.start = *at;
                right.duration = c.start + c.duration - at;
                c.duration = at - c.start;
                clips.push(right);
            }
        }
        R::TrackFlag { track, flag, value } => {
            if let Some(t) = tracks.iter_mut().find(|t| t.id == *track) {
                match flag {
                    ui::TrackFlag::Mute => t.muted = *value,
                    ui::TrackFlag::Solo => t.solo = *value,
                    ui::TrackFlag::Arm => t.armed = *value,
                    ui::TrackFlag::AutomationRead => t.automation_read = *value,
                }
            }
        }
        R::AddPoint { track, beat, value } => {
            let id = provisional_id(txn, tracks);
            if let Some(t) = tracks.iter_mut().find(|t| t.id == *track) {
                if let Some(p) = t.automation.iter_mut().find(|p| p.id == id) {
                    p.beat = *beat;
                    p.value = *value;
                } else {
                    t.automation
                        .push(ui::AutomationPoint::new(id, *beat, *value));
                }
                t.automation.sort_by(|a, b| a.beat.total_cmp(&b.beat));
            }
        }
        R::MovePoint {
            track,
            id,
            beat,
            value,
        } => {
            if let Some(t) = tracks.iter_mut().find(|t| t.id == *track) {
                if let Some(p) = t.automation.iter_mut().find(|p| p.id == *id) {
                    p.beat = *beat;
                    p.value = *value;
                }
                t.automation.sort_by(|a, b| a.beat.total_cmp(&b.beat));
            }
        }
        R::DeletePoint { track, id } => {
            if let Some(t) = tracks.iter_mut().find(|t| t.id == *track) {
                t.automation.retain(|p| p.id != *id);
            }
        }
    }
}
