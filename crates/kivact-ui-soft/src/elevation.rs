//! Role: Apply Neutral surface elevation to a style.
use crate::Tokens;
use kivact::style::Style;

/// Neutral surface elevation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Elevation {
    #[default]
    /// Keep the existing style without adding shadows; the default.
    None,
    /// Use the theme's low-elevation shadow layers.
    Low,
    /// Use the theme's medium-elevation shadow layers.
    Med,
    /// Use the theme's high-elevation shadow layers.
    High,
}

impl Elevation {
    pub(crate) fn apply(self, base: Style, tokens: &Tokens) -> Style {
        let shadows = match self {
            Self::None => return base,
            Self::Low => &tokens.shadows.low,
            Self::Med => &tokens.shadows.med,
            Self::High => &tokens.shadows.high,
        };
        shadows.iter().cloned().fold(base, Style::shadow)
    }
}

kivact::state::constant_value!(Elevation);
