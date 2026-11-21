//! Role: Hard-edged, information-dense operation-console components.
//!
//! The complete public API is built from the public `kivact` facade.
//! Install [`theme`] on an app or subtree (`theme(mode)`, with [`theme_with`]
//! and [`ThemeOptions`] for the density and the fonts), then compose the
//! constructors. The console's own variants are given with `.variant(..)`.
//!
//! The standard controls of `kivact::ui` (button, checkbox, switch, text
//! input, select, menu, dialog, tooltip, tabs, radio group, slider,
//! splitter and toast region) draw in the console's look under [`theme`].
//! The kit adds what the standard controls do not have.
//!
//! The color variables of the kit are in [`color`] and the font families in [`fonts`]; each
//! is a theme variable, so `theme.set(&color::PRIMARY_HOVER_VAR, ..)` changes it.

pub mod color;
pub mod fonts;
mod size;
mod variants;

mod choice_style;
mod icons;
mod recipes_collections;
mod recipes_controls;
mod recipes_controls_choice;
mod recipes_controls_progress;
mod recipes_feedback;
mod recipes_feedback_indicators;
mod recipes_input;
mod recipes_metrics;
mod recipes_navigation;
mod recipes_overlays;
mod shapes;
mod theme;
mod tone;

pub use theme::{Density, ThemeOptions, theme, theme_with};
pub use variants::{
    AvatarVariant, BadgeVariant, ButtonTone, ChoiceCard, GaugeSegments, HeadingVariant,
    InputGroupVariant, SkeletonVariant, Slant, StatVariant, Striped, TabsVariant,
};
