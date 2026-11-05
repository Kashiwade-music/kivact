//! Role: Interactive styled component catalog for kivact-ui-pop.
#![warn(redundant_imports)]
#![deny(clippy::too_many_lines, clippy::disallowed_types)]
mod fonts;
mod messages;
mod pop;

pub use fonts::fonts;
pub use messages::{locale, messages};
pub use pop::catalog;
