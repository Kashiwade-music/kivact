//! Role: Hard-edged, information-dense operation-console components.
//!
//! The complete public API is built from the public `kivact` facade.
//! Install [`theme`] on an app or subtree (`theme(mode)`, with
//! [`theme_with`] and [`ThemeOptions`] for the compact metrics), then compose the
//! constructors.
//! Constructors returning `Node` accept all ordinary Kivact modifiers.
//! Builders implement `NodeBuilder` and therefore `NodeBuilder` as well.
//!
//! The standard controls of `kivact::ui` (button, checkbox, switch, text
//! input, select, menu, dialog, tooltip, tabs, radio group, slider,
//! splitter and toast region) draw in the console's look under [`theme`];
//! [`Segmented`], [`ChoiceCard`], [`Navigation`] and [`Boxed`] are the
//! kit's other looks of the radio group and the tabs, given with
//! `.variant(..)`. The kit adds what the standard controls do not have.
//!
//! The texts the components show themselves (a dialog's close button, the
//! command palette's hint) are English and read through
//! `kivact::i18n::message`: an App replaces them with
//! `App::messages`.

mod button;
mod choice;
mod choice_style;
mod color;
mod display;
mod feedback;
mod icons;
mod input;
mod layout;
mod navigation;
mod overlays;
mod recipes;
mod recipes_collections;
mod recipes_field;
mod recipes_input;
mod recipes_palette;
mod recipes_small;
mod surface;
mod table;
mod theme;
mod tokens;

pub use button::{button_group, chevron, tool_button, toolbar};
pub use choice::{Boxed, ChoiceCard, Navigation, Segmented, card_label, navigation_label};
pub use color::ColorName;
pub use display::{
    ActivityEntry, Avatar, BadgeVariant, Gauge, Kpi, activity, avatar, avatar_more, avatar_stack,
    badge, gauge, kpi, property_list, ring, tag_badge,
};
pub use feedback::{Alert, Banner, EmptyState, Skeleton, alert, banner, empty_state, skeleton};
pub use input::{Dropzone, dropzone, input_adorned, input_group, input_group_content};
pub use layout::{TreeEntry, tree};
pub use navigation::{breadcrumb, pagination, stepper};
pub use overlays::{IconMenu, icon_menu};
pub use recipes::{ButtonTone, Slant};
pub use recipes_small::Striped;
pub use surface::{cut_corner, halo, hud, mono, section_heading};
pub use table::TableCell;
pub use theme::{ThemeOptions, theme, theme_with};
pub use tokens::{Density, Tokens};
