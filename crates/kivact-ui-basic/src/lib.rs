//! Role: shadcn/ui Lyra-styled components on the public Kivact facade.
//!
//! The look of the shadcn/ui "Lyra" preset: Neutral colors, square corners,
//! 12 px controls and Lucide icons. The only dependency is the public
//! `kivact` facade. Install [`theme`] on the App or a `kivact::style::provide_theme`
//! boundary: it sets the kit's color variables and the core variables that
//! draw every standard component of `kivact::ui` in this look.
//!
//! Messages are localized under the keys `basic.breadcrumb.more`,
//! `basic.chart.{bar, stacked_bar, line, area, pie, donut, radar, radial, summary,
//! total}` and `basic.questionnaire.{label, next, previous, progress, skip, submit}`.

mod attachment;
mod card_header;
mod chart;
mod code_block;
mod color;
mod field;
mod icons;
mod input_otp;
mod marker;
mod menubar;
mod native_select;
mod primitives;
mod questionnaire;
mod recipe_parts;
mod recipes_calendar;
mod recipes_chat;
mod recipes_code;
mod recipes_collections;
mod recipes_controls;
mod recipes_controls_button;
mod recipes_controls_check;
mod recipes_controls_field;
mod recipes_feedback;
mod recipes_feedback_avatar;
mod recipes_input;
mod recipes_metrics;
mod recipes_navigation;
mod recipes_navigation_sidebar;
mod recipes_overlays;
mod recipes_overlays_combobox;
mod recipes_overlays_command;
mod recipes_overlays_dialog;
mod scroll_area;
mod theme;
mod trace;
mod typography;
mod variants;

pub use attachment::{
    Attachment, AttachmentState, attachment, attachment_action, attachment_group,
};
pub use card_header::{CardHeader, card_header};
pub use chart::{Chart, ChartConfig, ChartKind, ChartRow, ChartSeries, chart};
pub use code_block::rust_lines;
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
pub use field::{FieldSeparator, FieldSet, field_content, field_group, field_separator, field_set};
pub use icons::ICON_NAMES;
pub use input_otp::{InputOtp, OtpPattern, input_otp};
pub use marker::{Marker, marker};
pub use menubar::{Menubar, menubar};
pub use native_select::{NativeSelect, NativeSelectSize, native_select};
pub use primitives::{
    Label, Separator, breadcrumb_ellipsis, item_separator, kbd_group, label, separator,
    table_caption,
};
pub use questionnaire::{
    Questionnaire, QuestionnaireAnswer, QuestionnaireChoice, QuestionnaireStep, questionnaire,
};
pub use scroll_area::{ScrollArea, scroll_area};
pub use theme::theme;
pub use trace::{TraceLine, trace, trace_collapsible, trace_line};
pub use typography::{blockquote, bullet_list, inline_code, large, lead, muted, paragraph, small};
pub use variants::{
    AlertDialogSize, AlertTone, AttachmentMediaVariant, AttachmentSize, BadgeVariant,
    CardHeaderBordered, CardSize, ChatBubbleVariant, ChatLayout, CodeBlockSize, DatePickerVariant,
    DisclosureVariant, DrawerVariant, EmptyMediaVariant, FieldLegendVariant, InputGroupButtonSize,
    InputGroupControl, ItemMediaVariant, ItemSize, ItemVariant, MarkerVariant, MenuVariant,
    TabsVariant, ToggleVariant,
};
