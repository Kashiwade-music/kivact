//! Role: Passive timeline tracks, tools, markers and automation values.

use kivact::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TrackFlag {
    Mute,
    Solo,
    Arm,
    AutomationRead,
}
impl TrackFlag {
    pub(crate) const ALL: [Self; 4] = [Self::Mute, Self::Solo, Self::Arm, Self::AutomationRead];
    pub(crate) fn checked(self, track: &TimelineTrack) -> bool {
        match self {
            Self::Mute => track.muted,
            Self::Solo => track.solo,
            Self::Arm => track.armed,
            Self::AutomationRead => track.automation_read,
        }
    }
}

/// The timeline's editing tool. Operations report application requests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimelineTool {
    #[default]
    Select,
    Split,
    Mute,
    Erase,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackKind {
    Video,
    Audio,
    Midi,
}
/// An editable automation knot, in beats and a normalized scalar value.
#[derive(Clone, Debug, PartialEq)]
pub struct AutomationPoint {
    pub id: u64,
    pub beat: f64,
    pub value: f64,
}
impl AutomationPoint {
    pub fn new(id: u64, beat: f64, value: f64) -> Self {
        Self { id, beat, value }
    }
}
/// Track header state and its optional automation lane.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineTrack {
    pub id: u64,
    pub name: String,
    pub kind: TrackKind,
    pub color: Color,
    // xtask-allow(state-fields): independent: Mute toggles muted; Solo, Arm and Automation Read leave it
    pub muted: bool,
    // xtask-allow(state-fields): independent: Solo toggles solo; Mute, Arm and Automation Read leave it
    pub solo: bool,
    // xtask-allow(state-fields): independent: Arm toggles armed; Mute, Solo and Automation Read leave it
    pub armed: bool,
    // xtask-allow(state-fields): independent: Automation Read toggles automation_read; Mute, Solo and Arm leave it
    pub automation_read: bool,
    pub automation: Vec<AutomationPoint>,
    /// Heading for the automation lane; empty hides the heading.
    pub automation_label: String,
}
impl TimelineTrack {
    pub fn new(id: u64, name: impl Into<String>, kind: TrackKind, color: Color) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            color,
            muted: false,
            solo: false,
            armed: false,
            automation_read: false,
            automation: Vec::new(),
            automation_label: String::new(),
        }
    }
    pub fn automation(mut self, points: Vec<AutomationPoint>) -> Self {
        self.automation = points;
        self.automation_read = true;
        self
    }
}
/// A ruler annotation. Prototype markers locate playback when their ruler is clicked.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineMarker {
    pub beat: f64,
    pub label: String,
    pub color: Color,
}
impl TimelineMarker {
    pub fn new(beat: f64, label: impl Into<String>, color: Color) -> Self {
        Self {
            beat,
            label: label.into(),
            color,
        }
    }
}

/// An absolute proposal for the application-owned arrangement model.
#[derive(Clone, Debug, PartialEq)]
pub enum ArrangementRequest {
    Retime(Vec<crate::editors::shared::model::ItemTime>),
    Delete(Vec<u64>),
    Split {
        clip: u64,
        at: f64,
    },
    Erase(u64),
    ToggleClipMute(u64),
    TrackFlag {
        track: u64,
        flag: TrackFlag,
        value: bool,
    },
    AddPoint {
        track: u64,
        beat: f64,
        value: f64,
    },
    MovePoint {
        track: u64,
        id: u64,
        beat: f64,
        value: f64,
    },
    DeletePoint {
        track: u64,
        id: u64,
    },
}
