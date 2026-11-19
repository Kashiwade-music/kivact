//! Role: Draw a standard sparkline, a small line chart of numbers, with no state.
//!
//! The chart is one node. Its box is the recipe's size and its line and area
//! are the theme's [`SparklineTokens`](crate::SparklineTokens); the values are
//! read where they are drawn, so a change of values redraws the line without
//! a state of the part. With a [`Sparkline::label`] it is an image named for
//! assistive technology with a description of its values; without one it is
//! decorative, which suits a statistic's chart slot.
pub(crate) mod look;
pub(crate) mod tokens;
mod view;

pub use view::{Sparkline, sparkline};
