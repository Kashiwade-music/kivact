//! Role: Dismiss a surface by swiping it toward its edge (a toast card, a drawer).
mod api;
mod machine;
mod types;

pub use api::{SwipeDismissApi, use_swipe_dismiss};
pub use types::{SwipeDismissPolicy, SwipeInputs, SwipePhase};
