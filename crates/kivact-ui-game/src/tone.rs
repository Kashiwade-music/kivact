//! Role: The console's color of each semantic tone, read from the theme it is painted in.
use kivact::interaction::Tone;
use kivact::style::{Color, Theme, color};

/// The colors a console theme gives a semantic [`Tone`].
pub(crate) trait ToneExt {
    /// The tone's solid color.
    fn color(self, t: &Theme) -> Color;
    /// The tone's weak tint, for backgrounds.
    fn weak(self, t: &Theme) -> Color;
}

impl ToneExt for Tone {
    fn color(self, t: &Theme) -> Color {
        match self {
            Self::Info => t.get(&color::ACCENT_VAR),
            Self::Success => t.get(&color::SUCCESS_VAR),
            Self::Warning => t.get(&color::WARNING_VAR),
            Self::Danger => t.get(&color::DANGER_VAR),
            Self::Neutral => t.get(&crate::color::TEXT_2_VAR),
            Self::Accent => t.get(&crate::color::YELLOW_VAR),
        }
    }
    fn weak(self, t: &Theme) -> Color {
        match self {
            Self::Info => t.get(&crate::color::PRIMARY_WEAK_VAR),
            Self::Success => t.get(&crate::color::SUCCESS_WEAK_VAR),
            Self::Warning => t.get(&crate::color::WARNING_WEAK_VAR),
            Self::Danger => t.get(&crate::color::DANGER_WEAK_VAR),
            Self::Neutral => t.get(&crate::color::SURFACE_3_VAR),
            Self::Accent => t.get(&crate::color::YELLOW_VAR),
        }
    }
}
