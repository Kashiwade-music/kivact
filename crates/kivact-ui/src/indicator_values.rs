//! Role: Define passive indicator states and theme metrics.
use crate::ControlSize;
use kivact_interaction::Tone;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonShape {
    #[default]
    Rect,
    Circle,
    Text,
    Rounded,
}
kivact_core::state::constant_value!(SkeletonShape);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarShape {
    #[default]
    Circle,
    Rounded,
    Square,
}
kivact_core::state::constant_value!(AvatarShape);
#[derive(Clone, Debug, PartialEq)]
pub struct BadgeState {
    pub tone: Tone,
    pub input: kivact_core::custom::PartState,
    // xtask-allow(state-fields): projection: readonly snapshot of the indicator's state for recipes, written only by its render function
    /// Whether the badge has an activation callback.
    pub actionable: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the indicator's state for recipes, written only by its render function
    pub leading_icon: bool,
    // xtask-allow(state-fields): projection: readonly snapshot of the indicator's state for recipes, written only by its render function
    pub trailing_icon: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct AvatarState {
    pub size: ControlSize,
    pub shape: AvatarShape,
    pub status: Option<Tone>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonState {
    pub shape: SkeletonShape,
    pub pulse: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SpinnerState {
    pub size: ControlSize,
    pub turn: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IndicatorTokens {
    pub period_ms: u64,
    /// Essential spinner cycle under reduced motion; absent keeps decorative motion paused.
    pub reduced_period_ms: Option<u64>,
    /// Optional skeleton base tween; index delay and alternating repetition are added by the component.
    pub skeleton_transition: Option<kivact_core::motion::Transition>,
    pub delay_ms: f64,
    pub stagger_ms: f64,
    pub line_height: f64,
    pub gap: f64,
    /// Logical pixels shared by adjacent avatars.
    pub avatar_overlap: f64,
}
impl Default for IndicatorTokens {
    fn default() -> Self {
        Self {
            period_ms: 1000,
            reduced_period_ms: None,
            skeleton_transition: None,
            delay_ms: 0.0,
            stagger_ms: 100.0,
            line_height: 16.0,
            gap: 6.0,
            avatar_overlap: 8.0,
        }
    }
}
