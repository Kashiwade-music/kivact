//! Role: A binding-backed cubic easing editor using shared drag edit sessions.

mod data;
pub(crate) mod geometry;
mod view;

pub use data::CubicBezier;
pub use view::{BezierCurve, bezier_curve};
