//! Role: The core corner radius variables of a theme (`radius::MD`), in logical px.

use kivact_types::{Length, ThemeVar};

use crate::space::length_vars;

length_vars! {
    /// A small corner radius.
    SM_VAR / SM = "radius.sm", 3.0;
    /// A medium corner radius.
    MD_VAR / MD = "radius.md", 6.0;
    /// A large corner radius.
    LG_VAR / LG = "radius.lg", 10.0;
}
