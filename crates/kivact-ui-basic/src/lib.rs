//! Role: shadcn/ui Lyra-styled components on the public Kivact facade.
//!
//! The look of the shadcn/ui "Lyra" preset: Neutral colors, square corners,
//! 12 px controls and Lucide icons. The only dependency is the public
//! `kivact` facade; behavior comes from `kivact::interaction` and the
//! standard `kivact::ui` components. A builder that draws one box accepts the
//! public [`kivact::view::NodeBuilder`] modifiers, and the style and
//! semantics it installs are only defaults: what the caller sets wins. A
//! dialog, an alert dialog, a sheet and a popover are the standard
//! `kivact::ui::dialog` of the kinds `Modal`, `AlertModal` (its width class
//! is [`AlertDialogSize`]), `Drawer(side)` and `Popover`, and a dropdown or
//! context menu is the standard `kivact::ui::menu` (`.attach(trigger)` or
//! `.attach_context(target)`), all in this look. Standard dialogs draw
//! their overlay away from the place they are used; a
//! standard `kivact::ui::hover_card` is attached to the caller's own trigger with
//! `.attach(trigger)`, which returns the trigger and the overlay.
//!
//! A slot method that places content in a component (`content`, `header`,
//! `footer`, `action`, `actions`, `media`, ...) takes `impl IntoView`; a
//! container's children (`column((..))`, `field(label, control)`) are views.
//!
//! Icons follow one rule. Every icon is drawn with the core
//! `kivact::view::icon("name")`, and [`theme`] puts every Lucide name of
//! [`ICON_NAMES`] in the theme's icon set. A slot that holds one view on a
//! single component takes `impl IntoView` (`alert(..).icon(icon("info"))`);
//! records, which are data rather than views (`ListItem`, `MenuEntry`,
//! `Command`), and the helper [`attachment_action`] take the name. Every per-item function argument is
//! `impl Fn(..) + 'static`.
//!
//! Install [`theme`] on the App or a `kivact::style::provide_theme`
//! boundary: it carries the typed [`Tokens`] and the recipes that draw every
//! standard component of `kivact::ui` (`button`, `toggle`, `checkbox`,
//! `switch`, `text_input`, `text_area`, `select`, `menu`, `dialog`, `tooltip`,
//! `tabs`, `radio_group`, `slider`, `range_slider`, `toast_region`, `splitter`,
//! `split`, `progress` and `disclosure`) in this look; the kit has no copies
//! of them. A few looks are values you give with `.variant(..)`, such as
//! [`ToggleVariant`] for a `toggle`. Docs import the crate as
//! `use kivact_ui_basic as basic;`.
//!
//! # Texts
//!
//! The texts the components show themselves (Loading, Close, Copy,
//! Copied, ...) are English and read through `kivact::i18n::message` under
//! unprefixed keys that the README lists; an App replaces them with
//! `App::messages`.

mod alert;
mod alert_dialog;
mod attachment;
mod avatar;
mod badge;
mod breadcrumb;
mod button;
mod card;
mod chart;
mod chat;
mod choice;
mod code_block;
mod color;
mod date_picker;
mod disclosure;
mod drawer;
mod empty;
mod field;
mod icons;
mod input_group;
mod input_otp;
mod item;
mod layers;
mod line_tabs;
mod list_popup;
mod marker;
mod menubar;
mod message_scroller;
mod native_select;
mod navigation_menu;
mod pagination;
mod parts;
mod primitives;
mod questionnaire;
mod recipes;
mod recipes_calendar;
mod recipes_collections;
mod recipes_combobox;
mod recipes_command;
mod recipes_controls;
mod recipes_dialog;
mod recipes_field;
mod recipes_groups;
mod recipes_menu;
mod recipes_metrics;
mod recipes_popup;
mod scroll_area;
mod sidebar;
mod table;
mod theme;
mod theme_access;
mod tokens;
mod trace;
mod typography;

pub use alert::AlertVariant;
pub use attachment::{
    Attachment, AttachmentMediaVariant, AttachmentSize, AttachmentState, attachment,
    attachment_action, attachment_group,
};
pub use badge::BadgeVariant;
pub use card::{CardLook, CardSize};
pub use chart::{Chart, ChartConfig, ChartKind, ChartRow, ChartSeries, chart};
pub use chat::{
    Bubble, BubbleAlign, BubbleReactionsSide, BubbleVariant, Message, MessageAlign, bubble,
    bubble_group, message, message_group,
};
pub use empty::EmptyMediaVariant;
pub use item::{ItemLook, ItemMediaVariant, ItemSize, ItemVariant};
pub use line_tabs::TabsVariant;
pub use marker::{Marker, MarkerVariant, marker};
pub use menubar::{Menubar, menubar};
pub use message_scroller::{MessageScroller, message_scroller};
pub use scroll_area::{ScrollArea, scroll_area};
pub use sidebar::{
    Sidebar, SidebarCollapsible, SidebarGroup, SidebarMenuButton, SidebarMenuButtonSize,
    SidebarMenuSubButton, SidebarSide, SidebarTrigger, SidebarVariant, sidebar, sidebar_group,
    sidebar_input, sidebar_menu, sidebar_menu_button, sidebar_menu_skeleton,
    sidebar_menu_sub_button, sidebar_provider, sidebar_separator, sidebar_trigger,
};

pub use alert_dialog::AlertDialogSize;
pub use choice::ToggleVariant;
pub use code_block::{
    CodeBlockLook, CodeBlockSize, CodeLine, CodeSpan, CodeTone, rust_lines, rust_source,
};
pub use color::ColorName;
pub use date_picker::DatePickerVariant;
pub use disclosure::DisclosureVariant;
pub use drawer::DrawerVariant;
pub use field::{
    FieldLegendVariant, FieldSeparator, FieldSet, field_content, field_group, field_separator,
    field_set,
};
pub use icons::ICON_NAMES;
pub use input_group::{InputGroupButtonLook, InputGroupButtonSize, InputGroupControl};
pub use input_otp::{InputOtp, OtpPattern, input_otp};
pub use native_select::{NativeSelect, NativeSelectSize, native_select};
pub use primitives::{Label, Separator, label, separator};
pub use questionnaire::{
    Questionnaire, QuestionnaireAnswer, QuestionnaireChoice, QuestionnaireStep, questionnaire,
};
pub use theme::theme;
pub use tokens::{CodeColors, Tokens};
pub use typography::{blockquote, bullet_list, inline_code, large, lead, muted, paragraph, small};
