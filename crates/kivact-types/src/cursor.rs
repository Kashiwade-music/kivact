//! Role: Shared mouse cursor appearance.

/// Mouse cursor appearance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Cursor {
    #[default]
    /// Auto cursor.
    Auto,
    /// Default cursor.
    Default,
    /// Text cursor.
    Text,
    /// Pointer cursor.
    Pointer,
    /// ColResize cursor.
    ColResize,
    /// RowResize cursor.
    RowResize,
    /// EwResize cursor.
    EwResize,
    /// NsResize cursor.
    NsResize,
    /// Grab cursor.
    Grab,
    /// Grabbing cursor.
    Grabbing,
    /// Crosshair cursor.
    Crosshair,
    /// Move cursor.
    Move,
    /// NotAllowed cursor.
    NotAllowed,
    /// None cursor.
    None,
}
