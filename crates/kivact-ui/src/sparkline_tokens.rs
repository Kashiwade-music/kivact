//! Role: Passive theme data for the sparkline's line, area and spacing.
use kivact_core::style::{Color, ColorToken};

/// The look of a standard sparkline. Configure it through [`crate::UiTokens`].
#[derive(Clone, Debug, PartialEq)]
pub struct SparklineTokens {
    /// The line color (default the theme's accent).
    pub line: Color,
    /// The line width in logical px (default 1.5).
    pub line_width: f64,
    /// Round caps and joins (default false).
    pub round: bool,
    /// The area under the line (default none).
    pub fill: SparklineFill,
    /// The vertical inset of the highest and lowest values in logical px (default 0).
    pub inset: f64,
}

/// The area under a sparkline's line, down to its bottom edge.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SparklineFill {
    /// No area.
    #[default]
    None,
    /// One solid color.
    Solid(Color),
    /// The color at the top of the box, fading to transparent at the bottom.
    Fade(Color),
}

impl Default for SparklineTokens {
    fn default() -> Self {
        Self {
            line: ColorToken::Accent.into(),
            line_width: 1.5,
            round: false,
            fill: SparklineFill::None,
            inset: 0.0,
        }
    }
}
