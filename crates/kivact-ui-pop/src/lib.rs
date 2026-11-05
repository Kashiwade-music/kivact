//! Role: Pop components for calm, compact desktop interfaces.
//!
//! The only dependency is the public `kivact` facade. Components use its
//! normal style, input, drawing, theme and overlay contracts. Every
//! builder accepts the public [`kivact::view::NodeBuilder`] modifiers.
//! Install [`theme`] or add [`Tokens`] to an existing Kivact theme.
//!
//! The kit's theme gives the standard components of `kivact::ui` the Pop
//! look, so tabs, radio groups, sliders and the toast region are
//! `kivact::ui::tabs`, `radio_group`, `slider` and `toast_region`; the kit
//! adds what they do not have ([`tab_label`] for a tab's count badge,
//! [`kivact::ui::RadioGroupVariant::Segmented`] for a segmented radio group,
//! the rest). Its theme also draws `kivact::ui::list` and
//! `kivact::ui::data_table`.
//!
//! The texts the components show themselves (a dialog's close button, the
//! command palette's hint) are English and read through
//! `kivact::i18n::message`: an App replaces them with
//! `App::messages`.

mod button;
mod chips;
mod choice;
mod color;
mod date;
mod display;
mod form;
mod icons;
mod messaging;
mod notifications;
mod patterns;
mod recipes;
mod recipes_collections;
mod recipes_extra;
mod recipes_field;
mod recipes_files;
mod recipes_groups;
mod recipes_indicators;
mod recipes_input;
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
pub use date::{
    Calendar, CalendarEvent, DateInput, DateRangeInput, calendar, date_input, date_range_input,
};
pub use display::{
    Alert, Avatar, Badge, BadgeVariant, EmptyState, ListItem, StatCard, alert, avatar, badge,
    empty_state, key_value, list_item, panel, progress_ring, section_heading, sparkline, stat_card,
};
pub use messaging::{
    Message, MessageGroup, Thread, message, message_group, thread, typing_message,
};
pub use notifications::{NotificationItem, NotificationMenu, notification_menu};
pub use patterns::{AgendaItem, Stepper, agenda_item, breadcrumb, stepper};
pub use recipes_indicators::SpinnerLook;
pub use recipes_small::{Segments, Striped};
pub use resize::resizable_text_area;
pub use tabs::tab_label;
pub use theme::theme;
pub use timeline::{TimelineItem, timeline_item};
pub use tokens::{Hue, Tokens};
pub use upload::upload_row;
