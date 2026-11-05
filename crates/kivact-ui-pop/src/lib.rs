//! Role: Pop components for calm, compact desktop interfaces.
//!
//! The only dependency is the public `kivact` facade. Components use its
//! normal style, input, drawing, theme and overlay contracts. Every
//! builder accepts the public [`kivact::view::NodeBuilder`] modifiers.
//! Install [`theme`] or add [`Tokens`] to an existing Kivact theme.
//!
//! The theme supplies the Pop appearance for standard controls, collections,
//! calendars, chat and metric components in `kivact::ui`. The kit adds typed
//! appearance variants, tab labels, chart primitives and domain-specific
//! conversation-preview and scheduling rows.
//!
//! The texts the components show themselves (a dialog's close button, the
//! command palette's hint) are English and read through
//! `kivact::i18n::message`: an App replaces them with
//! `App::messages`.

#![deny(clippy::too_many_lines, clippy::disallowed_types)]

mod button;
mod chips;
mod choice;
mod color;
mod display;
mod form;
mod icons;
mod messaging;
mod notifications;
mod patterns;
mod recipes;
mod recipes_avatar;
mod recipes_badge;
mod recipes_calendar;
mod recipes_chat;
mod recipes_collections;
mod recipes_controls;
mod recipes_extra;
mod recipes_feedback;
mod recipes_field;
mod recipes_files;
mod recipes_groups;
mod recipes_indicators;
mod recipes_input;
mod recipes_items;
mod recipes_metrics;
mod recipes_navigation;
mod recipes_palette;
mod recipes_small;
mod resize;
mod tabs;
mod theme;
mod timeline;
mod tokens;
mod upload;

pub use button::{ButtonTone, Slant};
pub use chips::{ChipList, RemovableChip, chip_list, removable_chip};
pub use color::ColorName;

pub use display::{empty_illustration, sparkline};
pub use messaging::{Thread, thread};
pub use notifications::{NotificationItem, NotificationMenu, notification_menu};
pub use patterns::{AgendaItem, agenda_item};
pub use recipes_avatar::AvatarLook;
pub use recipes_badge::{BadgeHue, BadgeVariant};
pub use recipes_calendar::WeekendHue;
pub use recipes_chat::PopBubble;
pub use recipes_indicators::SpinnerLook;
pub use recipes_items::ItemHue;
pub use recipes_metrics::{SectionNumber, StatHue};
pub use recipes_navigation::DiamondSteps;
pub use recipes_small::{ProgressHue, Segments, Striped};
pub use resize::resizable_text_area;
pub use tabs::tab_label;
pub use theme::theme;
pub use timeline::{TimelineItem, timeline_item};
pub use tokens::{Hue, Tokens};
pub use upload::upload_row;
