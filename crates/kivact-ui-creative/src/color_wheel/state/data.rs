//! Role: Shared control values and edit snapshots.
use super::super::geometry::Channel;
use kivact::style::Color;
pub(in crate::color_wheel) type Snapshot = (Color, f64, Option<Vec<Color>>);
/// Who began the active edit of the shared session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::color_wheel) enum ColorOwner {
    Wheel,
    Field { hsv: bool, index: usize },
    Hex,
    Swatch,
}
#[derive(Clone, Copy, PartialEq)]
pub(in crate::color_wheel) struct ColorControls {
    pub(in crate::color_wheel) channel: Channel,
    pub(in crate::color_wheel) swatch: SwatchSelection,
    pub(in crate::color_wheel) hue: f64,
}
#[derive(Clone, Copy, PartialEq)]
pub(in crate::color_wheel) enum SwatchSelection {
    None,
    Selected(usize),
}
impl SwatchSelection {
    pub(in crate::color_wheel) fn index(self) -> Option<usize> {
        match self {
            Self::None => None,
            Self::Selected(index) => Some(index),
        }
    }
}
