//! Role: Apply Neutral surface elevation to a style.
use crate::Tokens;
use kivact::style::Style;

/// Neutral surface elevation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Elevation {
    #[default]
    None,
    Low,
    Med,
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
