//! Role: Time item models and application edit requests.
use crate::KeyGlyph;
use crate::editors::shared::model::{Clip, ItemTime, drawable};
use kivact::style::{Color, rgb};
/// A key at a logical time; a positive duration draws an editable bar.
#[derive(Clone, Debug, PartialEq)]
pub struct Keyframe {
    pub id: u64,
    pub lane: u64,
    pub time: f64,
    pub duration: f64,
    pub label: String,
    pub color: Color,
    pub glyph: KeyGlyph,
    pub muted: bool,
}
impl Keyframe {
    pub fn new(id: u64, lane: u64, time: f64, label: impl Into<String>) -> Self {
        Self {
            id,
            lane,
            time,
            duration: 0.0,
            label: label.into(),
            color: rgb(0x8b7cff),
            glyph: KeyGlyph::Diamond,
            muted: false,
        }
    }
    pub fn duration(mut self, duration: f64) -> Self {
        self.duration = duration.max(0.0);
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    pub fn glyph(mut self, glyph: KeyGlyph) -> Self {
        self.glyph = glyph;
        self
    }
}
/// An application-owned change proposed by a timeline or dope sheet.
#[derive(Clone, Debug, PartialEq)]
pub enum TimeRequest {
    /// Move or trim, with absolute times and durations.
    Retime(Vec<ItemTime>),
    /// Delete or Backspace on the selection.
    Delete(Vec<u64>),
}

pub(crate) trait TimeItem: Clone + PartialEq + 'static {
    fn valid(&self) -> bool;
    fn id(&self) -> u64;
    fn lane(&self) -> u64;
    fn time(&self) -> f64;
    fn duration(&self) -> f64;
    fn label(&self) -> &str;
    fn color(&self) -> Color;
    fn muted(&self) -> bool;
    fn glyph(&self) -> KeyGlyph;
}
impl TimeItem for Clip {
    fn valid(&self) -> bool {
        drawable(self)
    }
    fn id(&self) -> u64 {
        self.id
    }
    fn lane(&self) -> u64 {
        self.lane
    }
    fn time(&self) -> f64 {
        self.start
    }
    fn duration(&self) -> f64 {
        self.duration
    }
    fn label(&self) -> &str {
        &self.label
    }
    fn color(&self) -> Color {
        self.color
    }
    fn muted(&self) -> bool {
        self.muted
    }
    fn glyph(&self) -> KeyGlyph {
        KeyGlyph::Square
    }
}
impl TimeItem for Keyframe {
    fn valid(&self) -> bool {
        self.time.is_finite() && self.duration.is_finite() && self.duration >= 0.0
    }
    fn id(&self) -> u64 {
        self.id
    }
    fn lane(&self) -> u64 {
        self.lane
    }
    fn time(&self) -> f64 {
        self.time
    }
    fn duration(&self) -> f64 {
        self.duration
    }
    fn label(&self) -> &str {
        &self.label
    }
    fn color(&self) -> Color {
        self.color
    }
    fn muted(&self) -> bool {
        self.muted
    }
    fn glyph(&self) -> KeyGlyph {
        self.glyph
    }
}
