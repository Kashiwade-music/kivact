//! Role: Shared control values and edit snapshots.
use super::super::data::GradientStop;
/// Who began the active edit of the shared session.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::gradient) enum GradientOwner {
    /// A drag of a stop on the ramp.
    Stop,
    /// The position field.
    Position,
    /// The color picker.
    Color,
    /// A whole step: adding, removing or moving a stop by key.
    Step,
}
pub(in crate::gradient) type Snapshot = (Vec<GradientStop>, Option<usize>);
