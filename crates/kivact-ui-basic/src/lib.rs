//! Role: shadcn/ui Lyra-styled components on the public Kivact facade.
//!
//! The look of the shadcn/ui "Lyra" preset: Neutral colors, square corners,
//! 12 px controls and Lucide icons. The only dependency is the public
//! `kivact` facade. Install [`theme`] on the App or a `kivact::style::provide_theme`
//! boundary: it sets the kit's color variables and the core variables that
//! draw every standard component of `kivact::ui` in this look.

mod color;
mod icons;
mod recipe_parts;
mod recipes_controls;
mod recipes_controls_button;
mod recipes_controls_check;
mod recipes_controls_field;
mod recipes_feedback;
mod theme;
mod variants;

pub use color::{
    ACCENT, ACCENT_FOREGROUND, ACCENT_FOREGROUND_VAR, ACCENT_VAR, BACKGROUND, BACKGROUND_VAR,
    BORDER, BORDER_VAR, BUBBLE_TINTED, BUBBLE_TINTED_HOVER, BUBBLE_TINTED_HOVER_VAR,
    BUBBLE_TINTED_VAR, CARD, CARD_FOREGROUND, CARD_FOREGROUND_VAR, CARD_VAR, CHART_1, CHART_1_VAR,
    CHART_2, CHART_2_VAR, CHART_3, CHART_3_VAR, CHART_4, CHART_4_VAR, CHART_5, CHART_5_VAR,
    DESTRUCTIVE, DESTRUCTIVE_VAR, FOREGROUND, FOREGROUND_VAR, INPUT, INPUT_VAR, MUTED,
    MUTED_FOREGROUND, MUTED_FOREGROUND_VAR, MUTED_VAR, POPOVER, POPOVER_FOREGROUND,
    POPOVER_FOREGROUND_VAR, POPOVER_VAR, PRIMARY, PRIMARY_FOREGROUND, PRIMARY_FOREGROUND_VAR,
    PRIMARY_VAR, RING, RING_VAR, SECONDARY, SECONDARY_FOREGROUND, SECONDARY_FOREGROUND_VAR,
    SECONDARY_VAR, SIDEBAR, SIDEBAR_ACCENT, SIDEBAR_ACCENT_FOREGROUND,
    SIDEBAR_ACCENT_FOREGROUND_VAR, SIDEBAR_ACCENT_VAR, SIDEBAR_BORDER, SIDEBAR_BORDER_VAR,
    SIDEBAR_FOREGROUND, SIDEBAR_FOREGROUND_VAR, SIDEBAR_PRIMARY, SIDEBAR_PRIMARY_FOREGROUND,
    SIDEBAR_PRIMARY_FOREGROUND_VAR, SIDEBAR_PRIMARY_VAR, SIDEBAR_RING, SIDEBAR_RING_VAR,
    SIDEBAR_VAR, SWITCH_THUMB, SWITCH_THUMB_VAR,
};
pub use icons::ICON_NAMES;
pub use theme::theme;
pub use variants::{
    AlertDialogSize, AlertTone, AttachmentMediaVariant, AttachmentSize, BadgeVariant,
    CardHeaderBordered, CardSize, ChatBubbleVariant, ChatLayout, CodeBlockSize, DatePickerVariant,
    DisclosureVariant, DrawerVariant, EmptyMediaVariant, FieldLegendVariant, InputGroupButtonSize,
    InputGroupControl, ItemMediaVariant, ItemSize, ItemVariant, MarkerVariant, MenuVariant,
    TabsVariant, ToggleVariant,
};
