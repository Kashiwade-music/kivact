//! Role: Running animations and the animation clock.

use serde::{Deserialize, Serialize};

use crate::common::{OwnerId, SourceLocation};

/// The kind of an animation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationKind {
    /// An animated value.
    Animated,
    /// A timeline.
    Timeline,
    /// A frame loop.
    FrameLoop,
}

/// One running animation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationInfo {
    /// The id of the animation.
    pub id: u64,
    /// The kind.
    pub kind: AnimationKind,
    /// Where the animation was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceLocation>,
    /// The owner of the animation.
    pub owner: OwnerId,
    /// Whether the animation is running.
    pub running: bool,
    /// Whether the animation is paused.
    pub paused: bool,
    /// Whether the animation is decorative.
    pub decorative: bool,
}

/// The running animations and the clock.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationList {
    /// The animations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub animations: Vec<AnimationInfo>,
    /// The scale of the animation clock.
    pub time_scale: f64,
}

/// The parameters of `set_time_scale`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeScaleParams {
    /// The scale: 0 pauses, 1 is normal, up to 4.
    pub scale: f64,
}
