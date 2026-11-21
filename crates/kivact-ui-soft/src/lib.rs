//! Role: Astryx Neutral as a Kivact UI kit: the look of every standard component, an app frame (shell, top and side navigation), and content parts (markdown, rich blocks, chat tools, tables).
//!
//! ```no_run
//! use kivact::prelude::*;
//! use kivact_ui_soft as soft;
//!
//! fn main() -> kivact::Result<()> {
//!     App::new("App").theme(soft::theme(ThemeMode::Light)).run(|| button("Save"))
//! }
//! ```
mod alert_dialog;
mod avatar;
mod avatar_status_dot;
mod banner;
mod blockquote;
mod card;
mod choice_list;
mod citation;
mod code;
mod color;
mod dialog_header;
mod divider;
mod feedback;
mod field_status;
mod form_layout;
mod grid;
mod hover_card;
mod icon;
mod icons;
mod indicator;
mod key_hints;
mod layout;
mod link;
mod media_query;
mod metrics;
mod motion;
mod radius;
mod recipe_parts;
mod recipes_calendar;
mod recipes_carousel;
mod recipes_chat;
mod recipes_code;
mod recipes_collections;
mod recipes_collections_table;
mod recipes_controls;
mod recipes_controls_choice;
mod recipes_controls_panel;
mod recipes_controls_radio;
mod recipes_controls_value;
mod recipes_feedback;
mod recipes_feedback_avatar;
mod recipes_feedback_banner;
mod recipes_feedback_empty;
mod recipes_field;
mod recipes_groups;
mod recipes_input;
mod recipes_metrics;
mod recipes_navigation;
mod recipes_navigation_pagination;
mod recipes_navigation_stepper;
mod recipes_overlays;
mod recipes_overlays_command;
mod recipes_overlays_dialog;
mod recipes_overlays_menu;
mod recipes_overlays_select;
mod section;
mod shadows;
mod space;
mod stack;
mod text;
mod theme;
mod theme_policy;
mod theme_scope;
mod token;
mod type_scale;
mod variants;
mod visually_hidden;

pub use alert_dialog::{AlertDialog, alert_dialog};
pub use avatar::{AvatarSilhouette, avatar_silhouette};
pub use avatar_status_dot::{AvatarStatus, avatar_status_dot};
pub use banner::{BannerIcon, banner_icon};
pub use blockquote::{Blockquote, BlockquoteText, blockquote, blockquote_text};
pub use card::with_card_insets;
pub use choice_list::{CheckboxList, checkbox_list, checkbox_list_item, radio_list};
pub use citation::{Citation, CitationSource, CitationVariant, citation};
pub use code::{Code, CodeColor, code};
pub use color::{
    ACCENT, ACCENT_HOVER, ACCENT_MUTED, BG_BODY, BG_CARD, BG_ERROR_INVERTED, BG_INVERTED, BG_MUTED,
    BG_POPOVER, BG_SURFACE, BORDER_EMPHASIZED, CHOICE_BORDER_HOVER, CHOICE_SURFACE_HOVER,
    DATA_BLUE, DATA_BLUE_RAMP_1, DATA_BLUE_RAMP_2, DATA_BLUE_RAMP_3, DATA_BLUE_RAMP_4,
    DATA_BLUE_RAMP_5, DATA_BROWN, DATA_CYAN, DATA_GRAY_RAMP_1, DATA_GRAY_RAMP_2, DATA_GRAY_RAMP_3,
    DATA_GRAY_RAMP_4, DATA_GRAY_RAMP_5, DATA_GREEN, DATA_INDIGO, DATA_NEUTRAL, DATA_ORANGE,
    DATA_ORANGE_RAMP_1, DATA_ORANGE_RAMP_2, DATA_ORANGE_RAMP_3, DATA_ORANGE_RAMP_4,
    DATA_ORANGE_RAMP_5, DATA_PINK, DATA_PINK_RAMP_1, DATA_PINK_RAMP_2, DATA_PINK_RAMP_3,
    DATA_PINK_RAMP_4, DATA_PINK_RAMP_5, DATA_PURPLE, DATA_PURPLE_RAMP_1, DATA_PURPLE_RAMP_2,
    DATA_PURPLE_RAMP_3, DATA_PURPLE_RAMP_4, DATA_PURPLE_RAMP_5, DATA_RED, DATA_RED_RAMP_1,
    DATA_RED_RAMP_2, DATA_RED_RAMP_3, DATA_RED_RAMP_4, DATA_RED_RAMP_5, DATA_SHAMROCK_RAMP_1,
    DATA_SHAMROCK_RAMP_2, DATA_SHAMROCK_RAMP_3, DATA_SHAMROCK_RAMP_4, DATA_SHAMROCK_RAMP_5,
    DATA_TEAL, DATA_TEAL_RAMP_1, DATA_TEAL_RAMP_2, DATA_TEAL_RAMP_3, DATA_TEAL_RAMP_4,
    DATA_TEAL_RAMP_5, DATA_YELLOW_RAMP_1, DATA_YELLOW_RAMP_2, DATA_YELLOW_RAMP_3,
    DATA_YELLOW_RAMP_4, DATA_YELLOW_RAMP_5, ERROR, ERROR_MUTED, HUE_BLUE_BG, HUE_BLUE_BORDER,
    HUE_BLUE_ICON, HUE_BLUE_TEXT, HUE_CYAN_BG, HUE_CYAN_BORDER, HUE_CYAN_ICON, HUE_CYAN_TEXT,
    HUE_GRAY_BG, HUE_GRAY_BORDER, HUE_GRAY_ICON, HUE_GRAY_TEXT, HUE_GREEN_BG, HUE_GREEN_BORDER,
    HUE_GREEN_ICON, HUE_GREEN_TEXT, HUE_ORANGE_BG, HUE_ORANGE_BORDER, HUE_ORANGE_ICON,
    HUE_ORANGE_TEXT, HUE_PINK_BG, HUE_PINK_BORDER, HUE_PINK_ICON, HUE_PINK_TEXT, HUE_PURPLE_BG,
    HUE_PURPLE_BORDER, HUE_PURPLE_ICON, HUE_PURPLE_TEXT, HUE_RED_BG, HUE_RED_BORDER, HUE_RED_ICON,
    HUE_RED_TEXT, HUE_TEAL_BG, HUE_TEAL_BORDER, HUE_TEAL_ICON, HUE_TEAL_TEXT, HUE_YELLOW_BG,
    HUE_YELLOW_BORDER, HUE_YELLOW_ICON, HUE_YELLOW_TEXT, ICON_ACCENT, ICON_DISABLED, ICON_PRIMARY,
    ICON_SECONDARY, INFO, INFO_MUTED, NEUTRAL, ON_ACCENT, ON_DARK, ON_ERROR, ON_ERROR_MUTED,
    ON_INFO, ON_INFO_MUTED, ON_LIGHT, ON_SUCCESS, ON_SUCCESS_MUTED, ON_WARNING, ON_WARNING_MUTED,
    OVERLAY_HOVER, OVERLAY_LIGHT_SCRIM, OVERLAY_PRESSED, OVERLAY_SCRIM, SHADOW, SKELETON, SUCCESS,
    SUCCESS_MUTED, SWITCH_OFF_HOVER, SYNTAX_ADDED, SYNTAX_ADDED_MARKER, SYNTAX_BACKGROUND,
    SYNTAX_DIFF_HEADER, SYNTAX_REMOVED, SYNTAX_REMOVED_MARKER, TEXT_ACCENT, TEXT_DISABLED,
    TEXT_ERROR, TEXT_INFO, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_SUCCESS, TEXT_WARNING, TINT_HOVER,
    TRACK, WARNING, WARNING_MUTED,
};
pub use dialog_header::{
    DialogHeader, DrawerHeader, EdgeCompensation, dialog_header, drawer_header,
};
pub use divider::{Divider, DividerVariant, divider};
pub use feedback::{StatusDot, StatusDotVariant, status_dot};
pub use field_status::{FieldStatus, FieldStatusVariant, StatusKind, field_status};
pub use form_layout::{
    FormDirection, FormLayout, FormLayoutCx, Optionality, form_layout, form_row,
};
pub use grid::{GridColumns, GridRepeat, LayoutGrid, layout_grid};
pub use hover_card::{HoverCardText, hover_card_text};
pub use icon::{IconColor, IconSize, IconView, icon_view};
pub use icons::ICON_NAMES;
pub use indicator::{
    CheckIndicator, CheckboxIndicator, RadioIndicator, check_indicator, checkbox_indicator,
    radio_indicator,
};
pub use key_hints::{keyboard_hint, with_key_hints};
pub use layout::{
    Layout, LayoutHeight, LayoutInset, LayoutRegion, layout, layout_content, layout_header,
    layout_panel,
};
pub use link::{Link, LinkColor, link};
pub use media_query::Breakpoint;
pub use radius::{
    RADIUS_CHAT, RADIUS_CONTAINER, RADIUS_ELEMENT, RADIUS_FULL, RADIUS_INNER, RADIUS_NONE,
    RADIUS_PAGE,
};
pub use section::{Section, SectionEdges, SectionVariant, section};
pub use space::{
    SPACE_CARD_PADDING, SPACE_DIALOG_PADDING, SPACE_LAYER_VIEWPORT_GUTTER, SPACE_S0, SPACE_S0_5,
    SPACE_S1, SPACE_S1_5, SPACE_S2, SPACE_S3, SPACE_S4, SPACE_S5, SPACE_S6, SPACE_S7, SPACE_S8,
    SPACE_S9, SPACE_S10, SPACE_S11, SPACE_S12, SPACE_SECTION_PADDING, Space,
};
pub use stack::{
    Center, CenterAxis, ContainerInset, Hstack, StackAlign, StackItem, StackItemSize, StackWrap,
    Vstack, center, hstack, stack_item, vstack,
};
pub use text::{
    StyledText, TextColor, TextJustify, TextKind, TextSize, TextWeight, TextWrap, TruncateTooltip,
    WordBreak, styled_text,
};
pub use theme::theme;
pub use theme_scope::{MediaMode, media_theme};
pub use token::{Token, TokenColor, token};
pub use variants::{
    AvatarGroupShape, AvatarSize, BadgeVariant, BannerVariant, BreadcrumbsVariant,
    ButtonGroupVariant, CardPadding, CardVariant, ChatBubbleVariant, ChatDensity,
    CodeBlockContainer, CodeBlockSize, CodeBlockWidth, ComposerElevation, DisclosureChevron,
    DisclosureDensity, DisclosureDividers, DropzoneVariant, Elevation, EmptyStateVariant,
    HeadingDisplay, ItemAlign, ItemDensity, ItemHighlighted, ItemLayout, LabelPosition,
    ListDividers, ListEdge, MetadataColumns, MetadataLabelWidth, MetadataOrientation,
    PaginationSize, RowStatusDot, SegmentedIconOnly, SegmentedLayout, SkeletonRadius, SpinnerShade,
    SpinnerVariant, StepCollapsedVariant, StepDensity, StepIndicatorPosition, SystemMessageVariant,
    TableCellAlign, TableCellDivider, TableRowDividers, TableRowHover, TableRowStriped,
    ToolbarVariant, TreeGuides,
};
pub use visually_hidden::{VisuallyHidden, visually_hidden};
