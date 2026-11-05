//! Role: Interactive styled component catalog for kivact-ui-game.
#![warn(redundant_imports)]
#![deny(clippy::too_many_lines, clippy::disallowed_types)]
mod fonts;
mod game;
mod messages;

pub use fonts::fonts;
pub use game::catalog;
pub use messages::{locale, messages};
