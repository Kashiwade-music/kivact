//! Role: Shared control values and edit snapshots.
/// What the hexadecimal field shows: the color, or the text typed since.
#[derive(Clone, PartialEq)]
pub(in crate::color_wheel) enum HexField {
    Following,
    Typed(String),
}
