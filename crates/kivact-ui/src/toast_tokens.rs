//! Role: Passive theme data for toast.
use kivact_core::motion::Transition;
/// A theme's optional toast row-height animation.
///
/// The region reads this at setup. With a transition, set the group
/// recipe's fixed gap to zero so the space between cards can animate.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToastRowMotion {
    /// A decorative transition for showing and collapsing the row.
    pub transition: Option<Transition>,
    /// Space before each card, included in its animated height.
    pub gap: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ToastTokens {
    pub row_motion: ToastRowMotion,
}
