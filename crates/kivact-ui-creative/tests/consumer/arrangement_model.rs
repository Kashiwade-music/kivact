//! Role: Application reducer and gesture snapshots for arrangement requests.
use kivact::state::{Signal, signal};
use kivact_ui_creative as ui;

pub(super) fn edits(
    clips: Signal<Vec<ui::Clip>>,
    tracks: Signal<Vec<ui::TimelineTrack>>,
    observe: impl Fn(kivact::view::EditPhase) + 'static,
) -> impl Fn(ui::EditorEvent<ui::ArrangementRequest>) + 'static {
    let snapshot = signal(None::<(Vec<ui::Clip>, Vec<ui::TimelineTrack>)>);
    let point = signal(None::<u64>);
    move |event| {
        use kivact::view::EditPhase;
        match event {
            ui::EditorEvent::Begin => {
                snapshot.set(Some((clips.get_untracked(), tracks.get_untracked())));
                point.set(None);
                observe(EditPhase::Begin);
            }
            ui::EditorEvent::Request(request) => {
                clips.update(|clips| tracks.update(|tracks| apply(clips, tracks, &request, point)));
            }
            ui::EditorEvent::Commit => {
                snapshot.set(None);
                point.set(None);
                observe(EditPhase::Commit);
            }
            ui::EditorEvent::Cancel => {
                if let Some((old_clips, old_tracks)) = snapshot.get_untracked() {
                    clips.set(old_clips);
                    tracks.set(old_tracks);
                }
                snapshot.set(None);
                point.set(None);
                observe(EditPhase::Cancel);
            }
        }
    }
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
    provisional: Signal<Option<u64>>,
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
            let id = provisional.get_untracked().unwrap_or_else(|| {
                let id = next_id(tracks.iter().flat_map(|t| &t.automation).map(|p| p.id));
                provisional.set(Some(id));
                id
            });
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

pub(super) fn arrangement(
    clips: Signal<Vec<ui::Clip>>,
    tracks: Signal<Vec<ui::TimelineTrack>>,
    state: ui::ArrangementState,
) -> ui::Arrangement {
    ui::arrangement(clips, tracks, state)
        .length(64.0)
        .on_edit(edits(clips, tracks, |_| {}))
}
