//! Role: Typed live targets for arrangement tracks, clips and automation.
use crate::Clip;
use crate::editors::arrangement::model::{TimelineTrack, TrackFlag};
use crate::editors::shared::targets::{self, Scene};
use kivact::draw::RegionId;
use kivact::{UiError, Value};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Target {
    Track(u64, TrackFlag),
    Automation(u64, u64),
    Item(u64),
    Trim(u64, bool),
    LoopCreate,
    LoopStart,
    LoopEnd,
    LoopBody,
    AddTrack,
    Playhead,
}
#[derive(Clone, Default, PartialEq)]
pub(crate) struct Regions(Option<Rc<Scene<Target>>>);
impl Regions {
    pub(crate) fn valid(&self) -> bool {
        self.0.is_some()
    }
    pub(crate) fn id(&self, target: Target) -> Option<u64> {
        self.0.as_ref()?.id(&target).map(|id| id.0)
    }
    pub(crate) fn target(&self, id: u64) -> Option<Target> {
        self.0.as_ref()?.target(RegionId(id)).copied()
    }
}
pub(crate) fn create(
    tracks: Value<Vec<TimelineTrack>>,
    clips: Value<Vec<Clip>>,
    selected: Value<Vec<u64>>,
) -> Value<Regions> {
    let scene = targets::targets(move || {
        let mut targets = vec![
            Target::LoopCreate,
            Target::LoopStart,
            Target::LoopEnd,
            Target::LoopBody,
            Target::AddTrack,
            Target::Playhead,
        ];
        let tracks = tracks.get();
        for track in &tracks {
            targets.extend(
                TrackFlag::ALL
                    .into_iter()
                    .map(|tag| Target::Track(track.id, tag)),
            );
            targets.extend(
                track
                    .automation
                    .iter()
                    .filter(|p| p.beat.is_finite() && p.value.is_finite())
                    .map(|p| Target::Automation(track.id, p.id)),
            );
        }
        for clip in clips.get() {
            if tracks.iter().any(|t| t.id == clip.lane)
                && crate::editors::shared::model::drawable(&clip)
            {
                targets.push(Target::Item(clip.id));
                if selected.get().contains(&clip.id) {
                    targets.extend([Target::Trim(clip.id, false), Target::Trim(clip.id, true)]);
                }
            }
        }
        Ok::<_, UiError>(targets)
    });
    kivact::memo(move || Regions(scene.get().ok()))
}
