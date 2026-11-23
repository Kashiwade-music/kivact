//! Role: Draw a standard sparkline, a small line chart of numbers.
//!
//! The chart is one node. Its box is the recipe's size, and its line and area
//! come from the recipe's paint, which reads the values in the part's state
//! and the theme variables `theme::sparkline::{LINE, LINE_WIDTH, INSET}` and
//! the settings of [`SparklineTheme`](crate::theme::SparklineTheme). With a
//! [`Sparkline::label`] it is an image named for assistive technology with a
//! description of its values; without one it is decorative, which suits a
//! statistic's chart slot.
pub(crate) mod look;
mod view;
pub use view::{Sparkline, sparkline};
