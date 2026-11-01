//! Role: Shared arrangement layout, geometry, painting and semantic regions.
use super::gesture::State;
use crate::editors::arrangement::model::{TimelineTrack, TrackKind};
use crate::editors::shared::metrics::{
    ARRANGEMENT_TOP, AUDIO_TRACK, AUTOMATION_LANE, AUTOMATION_PADDING, AUTOMATION_SPAN, VIDEO_TRACK,
};

pub(crate) struct TrackRow {
    pub(crate) id: u64,
    pub(crate) top: f32,
    pub(crate) height: f32,
    pub(crate) automation: Option<(f32, f32)>,
}
pub(crate) fn track_layout(tracks: &[TimelineTrack], offset_y: f32) -> Vec<TrackRow> {
    let mut top = ARRANGEMENT_TOP + offset_y;
    tracks
        .iter()
        .map(|track| {
            let height = if track.kind == TrackKind::Video {
                VIDEO_TRACK
            } else {
                AUDIO_TRACK
            };
            let automation =
                (!track.automation.is_empty()).then_some((top + height, AUTOMATION_LANE));
            let row = TrackRow {
                id: track.id,
                top,
                height,
                automation,
            };
            top += height + automation.map_or(0.0, |(_, height)| height);
            row
        })
        .collect()
}
pub(crate) fn automation_y(value: f64, top: f32) -> f32 {
    top + AUTOMATION_PADDING + (1.0 - value.clamp(0.0, 1.0)) as f32 * AUTOMATION_SPAN
}
pub(crate) fn automation_value(y: f32, top: f32) -> f64 {
    f64::from((1.0 - (y - top - AUTOMATION_PADDING) / AUTOMATION_SPAN).clamp(0.0, 1.0))
}
/// Clamp a vertical offset so the tracks stay within a viewport of `height`.
pub(crate) fn clamp_pan_y(tracks: &[TimelineTrack], height: f32, y: f32) -> f32 {
    let content = track_layout(tracks, 0.0)
        .iter()
        .map(|row| row.height + row.automation.map_or(0.0, |(_, h)| h))
        .sum::<f32>();
    let viewport = (height - ARRANGEMENT_TOP).max(0.0);
    y.clamp(-(content - viewport).max(0.0), 0.0)
}
pub(crate) fn pan_y(state: State, y: f32) -> f32 {
    state
        .track_data
        .with(|tracks| clamp_pan_y(tracks, state.size.get().height, y))
}
/// The vertical offset in effect. The stored one can lie outside the range
/// after tracks are removed or the viewport grows; the next pan writes a
/// clamped value back.
pub(crate) fn offset_y(state: State) -> f32 {
    pan_y(state, state.offset.get().y)
}
