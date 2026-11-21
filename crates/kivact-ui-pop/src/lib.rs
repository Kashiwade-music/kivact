//! Role: Pop components for calm, compact desktop interfaces.
//!
//! The only dependency is the public `kivact` facade. Components use its
//! normal style, input, drawing, theme and overlay contracts. Every
//! builder accepts the public [`kivact::view::Element`] modifiers.
//! Install [`theme`] to give the standard components of `kivact::ui` the Pop look.
//!
//! The theme supplies the Pop appearance for standard controls, collections,
//! calendars, chat and metric components in `kivact::ui`. The kit adds typed
//! appearance variants, tab labels and domain-specific
//! conversation-preview and scheduling rows.
//!
//! The texts the components show themselves (a dialog's close button, the
//! command palette's hint) are English and read through
//! `kivact::i18n::message`: an App replaces them with
//! `App::messages`.

mod icons;
mod recipe_parts;
mod recipes_calendar;
mod recipes_chat;
mod recipes_code;
mod recipes_collections;
mod recipes_controls;
mod recipes_controls_check;
mod recipes_controls_field;
mod recipes_feedback;
mod recipes_input;
mod recipes_metrics;
mod recipes_navigation;
mod recipes_overlays;
mod theme;
mod variants;

mod color;
mod size;

pub use theme::theme;
pub use variants::{
    AvatarHalo, AvatarHue, AvatarOnline, AvatarSize, BadgeVariant, ButtonTone, CalendarWeekdayHues,
    ChatBubbleVariant, DiamondSteps, Hue, SectionNumber, Segments, Slant, SpinnerVariant, Striped,
};
