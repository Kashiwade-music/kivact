//! Role: The variant types of the basic kit's standard components.

use kivact::Prop;

/// Colors of a standard alert callout. Chosen with `kivact::ui::alert(..).variant(AlertTone::Destructive)`;
/// other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum AlertTone {
    /// Card colors.
    #[default]
    Default,
    /// Destructive text and icon.
    Destructive,
}

/// The width class of a standard alert dialog. Chosen with
/// `kivact::ui::dialog(..).variant(AlertDialogSize::Sm)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum AlertDialogSize {
    /// 384 px wide; the media sits beside the text and the footer is right-aligned.
    #[default]
    Default,
    /// 320 px wide; everything is centered and the two buttons share the footer.
    Sm,
}

/// The size of an attachment (attachment.tsx `size`). Chosen with
/// `kivact::ui::attachment(..).variant(AttachmentSize::Xs)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum AttachmentSize {
    /// 40 px media, 8 px gap.
    #[default]
    Default,
    /// 32 px media.
    Sm,
    /// 28 px media.
    Xs,
}

/// The frame of the media of an attachment. Chosen with
/// `kivact::ui::attachment(..).variant(AttachmentMediaVariant::Image)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum AttachmentMediaVariant {
    /// An icon on a muted square.
    #[default]
    Icon,
    /// A picture that fills the square.
    Image,
}

/// The look of a standard `kivact::ui::badge` (badge.tsx `variant`). Chosen with
/// `kivact::ui::badge(..).variant(BadgeVariant::Outline)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum BadgeVariant {
    /// Solid primary.
    #[default]
    Default,
    /// Secondary fill.
    Secondary,
    /// Tinted destructive.
    Destructive,
    /// Bordered, no fill.
    Outline,
    /// No fill; muted on hover.
    Ghost,
    /// Primary text, underlined on hover.
    Link,
}

/// Spacing of a standard card. Chosen with `kivact::ui::card(..).variant(CardSize::Sm)`;
/// other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum CardSize {
    #[default]
    Default,
    Sm,
}

/// Draw the header rule of a standard card. Chosen with
/// `kivact::ui::card(..).variant(CardHeaderBordered(true))`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Prop)]
pub struct CardHeaderBordered(pub bool);

/// The Basic visual arrangement of a canonical chat message. Chosen with
/// `kivact::ui::chat_message(..).variant(ChatLayout::Bubble)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum ChatLayout {
    /// A full-width message row with optional author, avatar and metadata.
    #[default]
    Message,
    /// A standalone bubble sized to its contents.
    Bubble,
}

/// The Basic color treatment of a standard chat message content slot. Chosen with
/// `kivact::ui::chat_message(..).variant(ChatBubbleVariant::Secondary)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum ChatBubbleVariant {
    /// Solid primary: the sender's own text.
    #[default]
    Default,
    /// Secondary fill.
    Secondary,
    /// Muted fill.
    Muted,
    /// A faint gray derived from the primary color.
    Tinted,
    /// Bordered on the background.
    Outline,
    /// No fill, border or padding.
    Ghost,
    /// Tinted destructive.
    Destructive,
}

/// The two looks of a toggle: the variant of a `kivact::ui::toggle_button` and of
/// a standard `kivact::ui::toggle_group`. Chosen with `.variant(ToggleVariant::Outline)`;
/// other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum ToggleVariant {
    /// Transparent.
    #[default]
    Default,
    /// With a 1 px `input` border.
    Outline,
}

/// The size of a standard code block. Chosen with `kivact::ui::code_block(..).variant(CodeBlockSize::Sm)`;
/// other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum CodeBlockSize {
    #[default]
    Default,
    Sm,
}

/// The edge and spacing of a standard date field popup. Chosen with
/// `kivact::ui::date_input(..).variant(DatePickerVariant::Flush)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum DatePickerVariant {
    /// A calendar-sized surface without extra popup padding.
    #[default]
    Flush,
}

/// Visual treatment of a standard disclosure. Chosen with
/// `kivact::ui::disclosure(..).variant(DisclosureVariant::Trace)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum DisclosureVariant {
    /// Content without accordion padding, used with a caller-owned trigger.
    #[default]
    Plain,
    /// Muted activity detail below a compact caller-owned trigger.
    Trace,
}

/// The visual treatment of an edge dialog. Chosen with
/// `kivact::ui::dialog(..).variant(DrawerVariant::Sheet)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum DrawerVariant {
    /// A swipe drawer with its visible grab handle.
    #[default]
    Drawer,
    /// A sheet without the visible grab handle.
    Sheet,
}

/// The frame around a standard empty-state icon slot. Chosen with
/// `kivact::ui::empty_state(..).variant(EmptyMediaVariant::Icon)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum EmptyMediaVariant {
    /// Bare media.
    #[default]
    Default,
    /// A 32 px muted square around an icon.
    Icon,
}

/// The look of a field set legend. Chosen with
/// `kivact::ui::field_set(..).variant(FieldLegendVariant::Label)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum FieldLegendVariant {
    /// `text-sm`.
    #[default]
    Legend,
    /// `text-xs`, like a label.
    Label,
}

/// Draw a standard text input or text area without its own box inside a group. Chosen with
/// `kivact::ui::input_group(..).variant(InputGroupControl)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Prop)]
pub struct InputGroupControl;

/// Visual size of a canonical input-group button. Chosen with
/// `kivact::ui::button(..).variant(InputGroupButtonSize::Sm)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum InputGroupButtonSize {
    #[default]
    Xs,
    Sm,
    IconXs,
    IconSm,
}

/// The frame of an standard item row (item.tsx `variant`). Chosen with
/// `kivact::ui::item_list(..).variant(ItemVariant::Outline)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum ItemVariant {
    /// Transparent border.
    #[default]
    Default,
    /// Bordered.
    Outline,
    /// Muted fill.
    Muted,
}

/// The density of an standard item row (item.tsx `size`). Chosen with
/// `kivact::ui::item_list(..).variant(ItemSize::Sm)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum ItemSize {
    /// 12 px / 10 px padding.
    #[default]
    Default,
    /// The same padding with a smaller image.
    Sm,
}

/// The frame of the media slot of an standard item row (item.tsx media `variant`). Chosen with
/// `kivact::ui::item_list(..).variant(ItemMediaVariant::Icon)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum ItemMediaVariant {
    /// Bare content.
    #[default]
    Default,
    /// An icon.
    Icon,
    /// A 40 px (32 px for `Sm`) clipped picture.
    Image,
}

/// A look of `kivact::ui::tabs` beside the theme's filled one, given with
/// `.variant(..)`.
///
/// `Line` is tabs.tsx's `variant="line"`: no filled list, and a 2 px
/// underline in place of the selected tab's fill (beside the tab when
/// vertical). The tabs' keys, semantics and panels do not change:
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn reports() -> impl IntoView {
///     let current = signal("overview");
///     tabs(
///         "Report sections",
///         vec![ListItem::new("overview", "Overview"), ListItem::new("reports", "Reports")],
///         current,
///     )
///     .variant(basic::TabsVariant::Line)
///     .content(|key| text(key))
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Prop)]
pub enum TabsVariant {
    /// The underlined tabs of `variant="line"`.
    Line,
}

/// The decoration of a standard marker (marker.tsx `variant`). Chosen with
/// `kivact::ui::marker(..).variant(MarkerVariant::Separator)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum MarkerVariant {
    /// Plain muted text.
    #[default]
    Default,
    /// Rules fill the space on both sides of the content.
    Separator,
    /// A rule under the content.
    Border,
}

/// The kind of a kit menu, which the menu recipes read from the menu's
/// `.variant(..)`: a menu without one is a dropdown or a use_context menu.
/// Chosen with `kivact::ui::menu(..).variant(MenuVariant::Bar)`; other themes ignore it and draw the standard look.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Prop)]
pub enum MenuVariant {
    /// menubar.tsx: wider levels and check marks at the start.
    Bar,
}

impl kivact::ui::theme::Variant for AlertTone {}
impl kivact::ui::theme::Variant for AlertDialogSize {}
impl kivact::ui::theme::Variant for AttachmentSize {}
impl kivact::ui::theme::Variant for AttachmentMediaVariant {}
impl kivact::ui::theme::Variant for BadgeVariant {}
impl kivact::ui::theme::Variant for CardSize {}
impl kivact::ui::theme::Variant for CardHeaderBordered {}
impl kivact::ui::theme::Variant for ChatLayout {}
impl kivact::ui::theme::Variant for ChatBubbleVariant {}
impl kivact::ui::theme::Variant for ToggleVariant {}
impl kivact::ui::theme::Variant for CodeBlockSize {}
impl kivact::ui::theme::Variant for DatePickerVariant {}
impl kivact::ui::theme::Variant for DisclosureVariant {}
impl kivact::ui::theme::Variant for DrawerVariant {}
impl kivact::ui::theme::Variant for EmptyMediaVariant {}
impl kivact::ui::theme::Variant for FieldLegendVariant {}
impl kivact::ui::theme::Variant for InputGroupControl {}
impl kivact::ui::theme::Variant for InputGroupButtonSize {}
impl kivact::ui::theme::Variant for ItemVariant {}
impl kivact::ui::theme::Variant for ItemSize {}
impl kivact::ui::theme::Variant for ItemMediaVariant {}
impl kivact::ui::theme::Variant for TabsVariant {}
impl kivact::ui::theme::Variant for MarkerVariant {}
impl kivact::ui::theme::Variant for MenuVariant {}
