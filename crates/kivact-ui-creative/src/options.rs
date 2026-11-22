//! Role: The Creative kit's theme options: the accent, the density and the label tones.

use crate::color;
use kivact::style::Color;

/// The color of a label, resolved when drawn, so a label follows a theme
/// switch.
///
/// Application components resolve it with [`LabelTone::color`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum LabelTone {
    /// The ordinary label color.
    #[default]
    Normal,
    /// De-emphasized text.
    Muted,
    /// The accent color.
    Accent,
    /// A warning state.
    Warning,
    /// An error state.
    Error,
    /// A color chosen by the application, independent of the palette.
    Custom(Color),
}

impl LabelTone {
    /// The color this tone resolves to in the installed theme.
    pub fn color(self) -> Color {
        match self {
            Self::Normal => color::TEXT,
            Self::Muted => color::MUTED,
            Self::Accent => color::ACCENT_HIGH,
            Self::Warning => color::WARNING,
            Self::Error => color::X,
            Self::Custom(color) => color,
        }
    }
}

/// Control-row density, independent of display scale.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Density {
    #[default]
    Comfortable,
    Compact,
}
