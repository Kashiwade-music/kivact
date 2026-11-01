//! Role: Localized arrangement region names read during setup.
use crate::editors::shared::texts::Texts;
use kivact::Value;
use kivact::i18n::{interpolate, message};
/// The arrangement's texts (`arrangement.*`).
#[derive(Clone, Copy)]
pub(crate) struct ArrangementTexts {
    /// The texts the editors' items are drawn with.
    pub(crate) editor: Texts,
    playhead: Value<String>,
    loop_start: Value<String>,
    loop_end: Value<String>,
    loop_range: Value<String>,
    loop_ruler: Value<String>,
    add_track: Value<String>,
    tracks: Value<String>,
    automation_point: Value<String>,
    switches: [(Value<String>, Value<String>); 4],
}

impl ArrangementTexts {
    /// Read the texts. Setup only.
    #[track_caller]
    pub(crate) fn new() -> Self {
        Self {
            editor: Texts::new(),
            playhead: message("creative.editor.playhead", "Playhead"),
            loop_start: message("creative.arrangement.loop_start", "Loop start"),
            loop_end: message("creative.arrangement.loop_end", "Loop end"),
            loop_ruler: message("creative.arrangement.loop_ruler", "Loop ruler"),
            loop_range: message("creative.arrangement.loop_range", "Loop range"),
            add_track: message("creative.arrangement.add_track", "Add Track"),
            tracks: message("creative.arrangement.tracks", "{count} Tracks"),
            automation_point: message(
                "creative.arrangement.automation_point",
                "{track} automation point {id}",
            ),
            switches: [
                (
                    message("creative.arrangement.mute", "{track} Mute"),
                    message("creative.arrangement.mute_short", "M"),
                ),
                (
                    message("creative.arrangement.solo", "{track} Solo"),
                    message("creative.arrangement.solo_short", "S"),
                ),
                (
                    message("creative.arrangement.record", "{track} Record"),
                    message("creative.arrangement.record_short", "R"),
                ),
                (
                    message(
                        "creative.arrangement.read_automation",
                        "{track} Read Automation",
                    ),
                    message("creative.arrangement.read_automation_short", "A"),
                ),
            ],
        }
    }

    pub(crate) fn playhead(&self) -> String {
        self.playhead.get()
    }

    pub(crate) fn loop_start(&self) -> String {
        self.loop_start.get()
    }

    pub(crate) fn loop_end(&self) -> String {
        self.loop_end.get()
    }

    pub(crate) fn loop_ruler(&self) -> String {
        self.loop_ruler.get()
    }

    pub(crate) fn loop_range(&self) -> String {
        self.loop_range.get()
    }

    pub(crate) fn add_track(&self) -> String {
        self.add_track.get()
    }

    pub(crate) fn tracks(&self, count: usize) -> String {
        interpolate(&self.tracks.get(), &[("count", &count.to_string())])
    }

    pub(crate) fn automation_point(&self, track: &str, id: u64) -> String {
        interpolate(
            &self.automation_point.get(),
            &[("track", track), ("id", &id.to_string())],
        )
    }

    /// The name of switch `tag` (mute, solo, record, read automation) of
    /// `track`.
    pub(crate) fn switch(
        &self,
        tag: crate::editors::arrangement::model::TrackFlag,
        track: &str,
    ) -> String {
        interpolate(
            &self.switches[match tag {
                crate::editors::arrangement::model::TrackFlag::Mute => 0,
                crate::editors::arrangement::model::TrackFlag::Solo => 1,
                crate::editors::arrangement::model::TrackFlag::Arm => 2,
                crate::editors::arrangement::model::TrackFlag::AutomationRead => 3,
            }]
            .0
            .get(),
            &[("track", track)],
        )
    }

    /// The letter drawn on switch `tag`.
    pub(crate) fn switch_letter(
        &self,
        tag: crate::editors::arrangement::model::TrackFlag,
    ) -> String {
        self.switches[match tag {
            crate::editors::arrangement::model::TrackFlag::Mute => 0,
            crate::editors::arrangement::model::TrackFlag::Solo => 1,
            crate::editors::arrangement::model::TrackFlag::Arm => 2,
            crate::editors::arrangement::model::TrackFlag::AutomationRead => 3,
        }]
        .1
        .get()
    }
}
