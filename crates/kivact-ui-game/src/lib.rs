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
//! The color variables of the kit are in [`color`] and the font families in
//! [`fonts`]; each is a theme variable, so `theme.set(&color::PRIMARY_HOVER_VAR, ..)`
//! changes it.

pub mod color;
pub mod fonts;

mod button;
mod choice;
mod choice_style;
mod control_parts;
mod display;
mod icons;
mod kpi;
mod overlays;
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
mod size;
mod sparkline;
mod surface;
mod table;
mod theme;
mod tone;
mod variants;

pub use button::{chevron, tool_button};
pub use choice::{card_label, navigation_label};
pub use display::{ActivityEntry, activity};
pub use kpi::{Kpi, kpi};
pub use overlays::{IconMenu, icon_menu};
pub use shapes::cut_corner;
pub use size::Density;
pub use surface::{halo, hud, mono};
pub use table::{
    BadgeCell, DetailCell, IdentifierCell, ProgressCell, UserCell, badge_cell, detail_cell,
    identifier_cell, progress_cell, user_cell,
};
pub use theme::{ThemeOptions, theme, theme_with};
pub use variants::{
    AvatarVariant, BadgeVariant, ChoiceCard, GaugeSegments, HeadingVariant, InputGroupVariant,
    SkeletonVariant, StatVariant, TabsVariant,
};
