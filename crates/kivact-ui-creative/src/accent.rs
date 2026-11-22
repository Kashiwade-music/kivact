//! Role: The Creative kit's accent choices.

/// The five accent choices in the original creative-tool catalog.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Accent {
    #[default]
    Iris,
    Lime,
    Aqua,
    Coral,
    Amber,
}
