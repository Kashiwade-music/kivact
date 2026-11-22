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

mod avatar_decorations;
mod chips;
mod empty_illustration;
mod icons;
mod messaging;
mod notifications;
mod patterns;
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
mod resize;
mod tabs;
mod theme;
mod timeline;
mod upload;
mod variants;

mod color;
mod size;

pub use avatar_decorations::{AvatarDecorations, avatar_decorations};
pub use chips::{ChipList, RemovableChip, chip_list, removable_chip};
pub use color::{
    BLUE, BLUE_VAR, LINE, LINE_VAR, MINT, MINT_VAR, RED, RED_VAR, SURFACE, SURFACE_VAR, TEXT,
    TEXT_MUTED, TEXT_MUTED_VAR, TEXT_VAR, YELLOW, YELLOW_VAR,
};
pub use empty_illustration::empty_illustration;
pub use messaging::{Thread, thread};
pub use notifications::{NotificationItem, NotificationMenu, notification_menu};
pub use patterns::{AgendaItem, agenda_item};
pub use resize::resizable_text_area;
pub use tabs::tab_label;
pub use theme::theme;
pub use timeline::{TimelineItem, timeline_item};
pub use upload::upload_row;
pub use variants::{
    AvatarHue, AvatarSize, BadgeVariant, ButtonTone, CalendarWeekdayHues, ChatBubbleVariant,
    DiamondSteps, Hue, SectionNumber, Segments, SpinnerVariant,
};
