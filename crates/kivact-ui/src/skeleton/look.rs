//! Role: The skeleton recipe state, recipe table and default looks.

use crate::recipe::recipe_table;
use crate::theme::Look;
use kivact_core::style::{Length, ThemeVar};
use kivact_core::style::{Style, Theme, pct, style};
use kivact_macros::Prop;

/// The period of a skeleton's pulse, in milliseconds.
pub static PERIOD_MS_VAR: ThemeVar<f64> = ThemeVar::new("skeleton.period_ms", 1000.0);
/// [`PERIOD_MS_VAR`] as a length for a style.
pub const PERIOD_MS: Length = Length::var(&PERIOD_MS_VAR);

/// The delay before a skeleton's first pulse, in milliseconds.
pub static DELAY_MS_VAR: ThemeVar<f64> = ThemeVar::new("skeleton.delay_ms", 0.0);
/// [`DELAY_MS_VAR`] as a length for a style.
pub const DELAY_MS: Length = Length::var(&DELAY_MS_VAR);

/// The delay between the skeleton lines of a group, in milliseconds.
pub static STAGGER_MS_VAR: ThemeVar<f64> = ThemeVar::new("skeleton.stagger_ms", 100.0);
/// [`STAGGER_MS_VAR`] as a length for a style.
pub const STAGGER_MS: Length = Length::var(&STAGGER_MS_VAR);

/// The height of a skeleton line, in logical px.
pub static LINE_HEIGHT_VAR: ThemeVar<f64> = ThemeVar::new("skeleton.line_height", 16.0);
/// [`LINE_HEIGHT_VAR`] as a length for a style.
pub const LINE_HEIGHT: Length = Length::var(&LINE_HEIGHT_VAR);

/// The gap between the skeleton lines, in logical px.
pub static GAP_VAR: ThemeVar<f64> = ThemeVar::new("skeleton.gap", 6.0);
/// [`GAP_VAR`] as a length for a style.
pub const GAP: Length = Length::var(&GAP_VAR);

pub(crate) fn skeleton(t: &Theme, s: &Look<SkeletonState>) -> Style {
    let base = style()
        .min_height(LINE_HEIGHT)
        .width(pct(100))
        .background(kivact_core::style::color::BORDER)
        .opacity(s.pulse);
    match s.shape {
        SkeletonShape::Circle => base.size(36, 36).border_radius(999),
        SkeletonShape::Rounded => base.border_radius(t.get(&kivact_core::style::radius::MD_VAR)),
        SkeletonShape::Rect => base.border_radius(0),
        SkeletonShape::Text => base.border_radius(t.get(&kivact_core::style::radius::SM_VAR)),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonState {
    pub shape: SkeletonShape,
    pub pulse: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum SkeletonShape {
    #[default]
    Rect,
    Circle,
    Text,
    Rounded,
}

recipe_table! {
    /// The recipes of a skeleton placeholder.
    SkeletonTheme {
        root(root_recipe): SkeletonState = skeleton,
    }
    settings {
        /// The base tween of the pulse; `None` tweens over the period.
        transition(transition_value): Option<kivact_core::motion::Transition> = None,
    }
}
