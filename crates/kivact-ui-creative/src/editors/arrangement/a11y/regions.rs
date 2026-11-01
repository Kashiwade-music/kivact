//! Role: Projects arrangement state into the accessible regions of its canvas.

use super::super::gesture::State;
use super::targets::Target;
use crate::editors::arrangement::model::TrackFlag;
use crate::editors::shared::model::drawable;
use crate::editors::shared::region_parts::{Kind, RegionInfo, bounded_range};
use kivact::draw::RegionId;

pub(crate) fn semantic(state: State) -> Vec<RegionInfo> {
    let mut result = Vec::new();
    let selected = state.selection.get();
    let data = state.data.get();
    let tracks = state.track_data.get();
    let regions = state.regions.get();
    if !regions.valid() {
        return result;
    }
    for (target, kind, name) in [
        (
            Target::LoopCreate,
            Kind::LoopCreate,
            state.texts.loop_ruler(),
        ),
        (
            Target::LoopStart,
            Kind::LoopEndpoint,
            state.texts.loop_start(),
        ),
        (Target::LoopEnd, Kind::LoopEndpoint, state.texts.loop_end()),
        (Target::LoopBody, Kind::LoopBody, state.texts.loop_range()),
        (Target::AddTrack, Kind::AddTrack, state.texts.add_track()),
        (
            Target::Playhead,
            Kind::ArrangementPlayhead,
            state.texts.playhead(),
        ),
    ] {
        if let Some(id) = regions.id(target) {
            let mut info = RegionInfo::new(RegionId(id), kind, name);
            let (a, b) = bounded_range(
                state.range.get(),
                state.length.get(),
                state.loop_min_duration,
            );
            info.now = match target {
                Target::LoopStart => a,
                Target::LoopEnd => b,
                _ => state.frame.get(),
            };
            info.max = state.length.get();
            result.push(info);
        }
    }
    for track in &tracks {
        for tag in TrackFlag::ALL {
            let checked = tag.checked(track);
            if let Some(id) = regions.id(Target::Track(track.id, tag)) {
                let mut info = RegionInfo::new(
                    RegionId(id),
                    Kind::TrackSwitch,
                    state.texts.switch(tag, &track.name),
                );
                info.checked = checked;
                result.push(info);
            }
        }
        for clip in data.iter().filter(|c| c.lane == track.id && drawable(c)) {
            let Some(id) = regions.id(Target::Item(clip.id)) else {
                continue;
            };
            let selected = selected.contains(&clip.id);
            let mut info = RegionInfo::new(RegionId(id), Kind::Item, clip.label.clone());
            info.value = state.texts.editor.selection(selected);
            result.push(info);
            if selected {
                for right in [false, true] {
                    if let Some(id) = regions.id(Target::Trim(clip.id, right)) {
                        result.push(RegionInfo::new(
                            RegionId(id),
                            Kind::Trim,
                            state.texts.editor.trim(right, &clip.label),
                        ));
                    }
                }
            }
        }
        for knot in track
            .automation
            .iter()
            .filter(|p| p.beat.is_finite() && p.value.is_finite())
        {
            if let Some(id) = regions.id(Target::Automation(track.id, knot.id)) {
                let mut info = RegionInfo::new(
                    RegionId(id),
                    Kind::Automation,
                    state.texts.automation_point(&track.name, knot.id),
                );
                info.min = 0.0;
                info.max = 1.0;
                info.now = knot.value;
                result.push(info);
            }
        }
    }
    result
}
