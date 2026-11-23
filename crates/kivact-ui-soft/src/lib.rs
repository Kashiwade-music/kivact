//! Role: Astryx Neutral as a Kivact UI kit: the look of every standard component, an app
//! frame (shell, top and side navigation), and content parts (markdown, rich
//! blocks, chat tools, tables).
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
mod app_shell;
mod avatar;
mod avatar_status_dot;
mod banner;
mod blockquote;
mod bottom_sheet;
mod card;
mod chat_dictation;
mod choice_list;
mod citation;
mod code;
mod code_syntax;
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
mod lightbox;
mod link;
mod markdown;
mod markdown_parser;
mod media_overlay;
mod media_query;
mod metrics;
mod mobile_nav;
mod more_menu;
mod motion;
mod nav_heading;
mod nav_icon;
mod navigation_session;
mod outline;
mod pagination;
mod power_search;
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
mod recipes_navigation_sidebar;
mod recipes_navigation_stepper;
mod recipes_overlays;
mod recipes_overlays_command;
mod recipes_overlays_dialog;
mod recipes_overlays_menu;
mod recipes_overlays_select;
mod rich_block_doc;
mod rich_blocks;
mod rich_text;
mod scrollable_area;
mod section;
mod shadows;
mod soft_chat_tools;
mod soft_overflow;
mod soft_table;
mod space;
mod stack;
mod text;
mod theme;
mod theme_policy;
mod theme_scope;
mod thumbnail;
mod timer;
mod timestamp;
mod token;
mod tokens;
mod top_nav;
mod type_scale;
mod variants;
mod visually_hidden;

pub use alert_dialog::{AlertDialog, alert_dialog};
pub use app_shell::{MobileNavConfig, MobileNavMode, ShellHeight, ShellVariant, app_shell};
pub use avatar::{AvatarSilhouette, avatar_silhouette};
pub use avatar_status_dot::{AvatarStatus, avatar_status_dot};
pub use banner::{BannerIcon, banner_icon};
pub use blockquote::{Blockquote, BlockquoteText, blockquote, blockquote_text};
pub use bottom_sheet::{
    BottomSheet, DialogPurpose, SheetHeight, SnapPoint, bottom_sheet, bottom_sheet_flow,
};
pub use card::with_card_insets;
pub use chat_dictation::{ChatDictationButton, DictationState, chat_dictation_button};
pub use choice_list::{CheckboxList, checkbox_list, checkbox_list_item, radio_list};
pub use citation::{Citation, CitationSource, CitationVariant, citation};
pub use code::{Code, CodeColor, code};
pub use code_syntax::{CodeLanguage, CodeLanguageInput, code_highlight};
pub use color::{
    ACCENT, ACCENT_HOVER, ACCENT_HOVER_VAR, ACCENT_MUTED, ACCENT_MUTED_VAR, ACCENT_VAR, BG_BODY,
    BG_BODY_VAR, BG_CARD, BG_CARD_VAR, BG_ERROR_INVERTED, BG_ERROR_INVERTED_VAR, BG_INVERTED,
    BG_INVERTED_VAR, BG_MUTED, BG_MUTED_VAR, BG_POPOVER, BG_POPOVER_VAR, BG_SURFACE,
    BG_SURFACE_VAR, BORDER_EMPHASIZED, BORDER_EMPHASIZED_VAR, BORDER_VAR, CHOICE_BORDER_HOVER,
    CHOICE_BORDER_HOVER_VAR, CHOICE_SURFACE_HOVER, CHOICE_SURFACE_HOVER_VAR, DATA_BLUE,
    DATA_BLUE_RAMP_1, DATA_BLUE_RAMP_1_VAR, DATA_BLUE_RAMP_2, DATA_BLUE_RAMP_2_VAR,
    DATA_BLUE_RAMP_3, DATA_BLUE_RAMP_3_VAR, DATA_BLUE_RAMP_4, DATA_BLUE_RAMP_4_VAR,
    DATA_BLUE_RAMP_5, DATA_BLUE_RAMP_5_VAR, DATA_BLUE_VAR, DATA_BROWN, DATA_BROWN_VAR, DATA_CYAN,
    DATA_CYAN_VAR, DATA_GRAY_RAMP_1, DATA_GRAY_RAMP_1_VAR, DATA_GRAY_RAMP_2, DATA_GRAY_RAMP_2_VAR,
    DATA_GRAY_RAMP_3, DATA_GRAY_RAMP_3_VAR, DATA_GRAY_RAMP_4, DATA_GRAY_RAMP_4_VAR,
    DATA_GRAY_RAMP_5, DATA_GRAY_RAMP_5_VAR, DATA_GREEN, DATA_GREEN_VAR, DATA_INDIGO,
    DATA_INDIGO_VAR, DATA_NEUTRAL, DATA_NEUTRAL_VAR, DATA_ORANGE, DATA_ORANGE_RAMP_1,
    DATA_ORANGE_RAMP_1_VAR, DATA_ORANGE_RAMP_2, DATA_ORANGE_RAMP_2_VAR, DATA_ORANGE_RAMP_3,
    DATA_ORANGE_RAMP_3_VAR, DATA_ORANGE_RAMP_4, DATA_ORANGE_RAMP_4_VAR, DATA_ORANGE_RAMP_5,
    DATA_ORANGE_RAMP_5_VAR, DATA_ORANGE_VAR, DATA_PINK, DATA_PINK_RAMP_1, DATA_PINK_RAMP_1_VAR,
    DATA_PINK_RAMP_2, DATA_PINK_RAMP_2_VAR, DATA_PINK_RAMP_3, DATA_PINK_RAMP_3_VAR,
    DATA_PINK_RAMP_4, DATA_PINK_RAMP_4_VAR, DATA_PINK_RAMP_5, DATA_PINK_RAMP_5_VAR, DATA_PINK_VAR,
    DATA_PURPLE, DATA_PURPLE_RAMP_1, DATA_PURPLE_RAMP_1_VAR, DATA_PURPLE_RAMP_2,
    DATA_PURPLE_RAMP_2_VAR, DATA_PURPLE_RAMP_3, DATA_PURPLE_RAMP_3_VAR, DATA_PURPLE_RAMP_4,
    DATA_PURPLE_RAMP_4_VAR, DATA_PURPLE_RAMP_5, DATA_PURPLE_RAMP_5_VAR, DATA_PURPLE_VAR, DATA_RED,
    DATA_RED_RAMP_1, DATA_RED_RAMP_1_VAR, DATA_RED_RAMP_2, DATA_RED_RAMP_2_VAR, DATA_RED_RAMP_3,
    DATA_RED_RAMP_3_VAR, DATA_RED_RAMP_4, DATA_RED_RAMP_4_VAR, DATA_RED_RAMP_5,
    DATA_RED_RAMP_5_VAR, DATA_RED_VAR, DATA_SHAMROCK_RAMP_1, DATA_SHAMROCK_RAMP_1_VAR,
    DATA_SHAMROCK_RAMP_2, DATA_SHAMROCK_RAMP_2_VAR, DATA_SHAMROCK_RAMP_3, DATA_SHAMROCK_RAMP_3_VAR,
    DATA_SHAMROCK_RAMP_4, DATA_SHAMROCK_RAMP_4_VAR, DATA_SHAMROCK_RAMP_5, DATA_SHAMROCK_RAMP_5_VAR,
    DATA_TEAL, DATA_TEAL_RAMP_1, DATA_TEAL_RAMP_1_VAR, DATA_TEAL_RAMP_2, DATA_TEAL_RAMP_2_VAR,
    DATA_TEAL_RAMP_3, DATA_TEAL_RAMP_3_VAR, DATA_TEAL_RAMP_4, DATA_TEAL_RAMP_4_VAR,
    DATA_TEAL_RAMP_5, DATA_TEAL_RAMP_5_VAR, DATA_TEAL_VAR, DATA_YELLOW_RAMP_1,
    DATA_YELLOW_RAMP_1_VAR, DATA_YELLOW_RAMP_2, DATA_YELLOW_RAMP_2_VAR, DATA_YELLOW_RAMP_3,
    DATA_YELLOW_RAMP_3_VAR, DATA_YELLOW_RAMP_4, DATA_YELLOW_RAMP_4_VAR, DATA_YELLOW_RAMP_5,
    DATA_YELLOW_RAMP_5_VAR, ERROR, ERROR_MUTED, ERROR_MUTED_VAR, ERROR_VAR, FOCUS_RING_VAR,
    HUE_BLUE_BG, HUE_BLUE_BG_VAR, HUE_BLUE_BORDER, HUE_BLUE_BORDER_VAR, HUE_BLUE_ICON,
    HUE_BLUE_ICON_VAR, HUE_BLUE_TEXT, HUE_BLUE_TEXT_VAR, HUE_CYAN_BG, HUE_CYAN_BG_VAR,
    HUE_CYAN_BORDER, HUE_CYAN_BORDER_VAR, HUE_CYAN_ICON, HUE_CYAN_ICON_VAR, HUE_CYAN_TEXT,
    HUE_CYAN_TEXT_VAR, HUE_GRAY_BG, HUE_GRAY_BG_VAR, HUE_GRAY_BORDER, HUE_GRAY_BORDER_VAR,
    HUE_GRAY_ICON, HUE_GRAY_ICON_VAR, HUE_GRAY_TEXT, HUE_GRAY_TEXT_VAR, HUE_GREEN_BG,
    HUE_GREEN_BG_VAR, HUE_GREEN_BORDER, HUE_GREEN_BORDER_VAR, HUE_GREEN_ICON, HUE_GREEN_ICON_VAR,
    HUE_GREEN_TEXT, HUE_GREEN_TEXT_VAR, HUE_ORANGE_BG, HUE_ORANGE_BG_VAR, HUE_ORANGE_BORDER,
    HUE_ORANGE_BORDER_VAR, HUE_ORANGE_ICON, HUE_ORANGE_ICON_VAR, HUE_ORANGE_TEXT,
    HUE_ORANGE_TEXT_VAR, HUE_PINK_BG, HUE_PINK_BG_VAR, HUE_PINK_BORDER, HUE_PINK_BORDER_VAR,
    HUE_PINK_ICON, HUE_PINK_ICON_VAR, HUE_PINK_TEXT, HUE_PINK_TEXT_VAR, HUE_PURPLE_BG,
    HUE_PURPLE_BG_VAR, HUE_PURPLE_BORDER, HUE_PURPLE_BORDER_VAR, HUE_PURPLE_ICON,
    HUE_PURPLE_ICON_VAR, HUE_PURPLE_TEXT, HUE_PURPLE_TEXT_VAR, HUE_RED_BG, HUE_RED_BG_VAR,
    HUE_RED_BORDER, HUE_RED_BORDER_VAR, HUE_RED_ICON, HUE_RED_ICON_VAR, HUE_RED_TEXT,
    HUE_RED_TEXT_VAR, HUE_TEAL_BG, HUE_TEAL_BG_VAR, HUE_TEAL_BORDER, HUE_TEAL_BORDER_VAR,
    HUE_TEAL_ICON, HUE_TEAL_ICON_VAR, HUE_TEAL_TEXT, HUE_TEAL_TEXT_VAR, HUE_YELLOW_BG,
    HUE_YELLOW_BG_VAR, HUE_YELLOW_BORDER, HUE_YELLOW_BORDER_VAR, HUE_YELLOW_ICON,
    HUE_YELLOW_ICON_VAR, HUE_YELLOW_TEXT, HUE_YELLOW_TEXT_VAR, ICON_ACCENT, ICON_ACCENT_VAR,
    ICON_DISABLED, ICON_DISABLED_VAR, ICON_PRIMARY, ICON_PRIMARY_VAR, ICON_SECONDARY,
    ICON_SECONDARY_VAR, INFO, INFO_MUTED, INFO_MUTED_VAR, INFO_VAR, NEUTRAL, NEUTRAL_VAR,
    ON_ACCENT, ON_ACCENT_VAR, ON_DARK, ON_DARK_VAR, ON_ERROR, ON_ERROR_MUTED, ON_ERROR_MUTED_VAR,
    ON_ERROR_VAR, ON_INFO, ON_INFO_MUTED, ON_INFO_MUTED_VAR, ON_INFO_VAR, ON_LIGHT, ON_LIGHT_VAR,
    ON_SUCCESS, ON_SUCCESS_MUTED, ON_SUCCESS_MUTED_VAR, ON_SUCCESS_VAR, ON_WARNING,
    ON_WARNING_MUTED, ON_WARNING_MUTED_VAR, ON_WARNING_VAR, OVERLAY_HOVER, OVERLAY_HOVER_VAR,
    OVERLAY_LIGHT_SCRIM, OVERLAY_LIGHT_SCRIM_VAR, OVERLAY_PRESSED, OVERLAY_PRESSED_VAR,
    OVERLAY_SCRIM, OVERLAY_SCRIM_VAR, SHADOW, SHADOW_VAR, SKELETON, SKELETON_VAR, SUCCESS,
    SUCCESS_MUTED, SUCCESS_MUTED_VAR, SUCCESS_VAR, SWITCH_OFF_HOVER, SWITCH_OFF_HOVER_VAR,
    SYNTAX_ADDED, SYNTAX_ADDED_MARKER, SYNTAX_ADDED_MARKER_VAR, SYNTAX_ADDED_VAR,
    SYNTAX_BACKGROUND, SYNTAX_BACKGROUND_VAR, SYNTAX_DIFF_HEADER, SYNTAX_DIFF_HEADER_VAR,
    SYNTAX_REMOVED, SYNTAX_REMOVED_MARKER, SYNTAX_REMOVED_MARKER_VAR, SYNTAX_REMOVED_VAR,
    TEXT_ACCENT, TEXT_ACCENT_VAR, TEXT_DISABLED, TEXT_DISABLED_VAR, TEXT_ERROR, TEXT_ERROR_VAR,
    TEXT_INFO, TEXT_INFO_VAR, TEXT_PRIMARY, TEXT_PRIMARY_VAR, TEXT_SECONDARY, TEXT_SECONDARY_VAR,
    TEXT_SUCCESS, TEXT_SUCCESS_VAR, TEXT_WARNING, TEXT_WARNING_VAR, TINT_HOVER, TINT_HOVER_VAR,
    TRACK, TRACK_VAR, WARNING, WARNING_MUTED, WARNING_MUTED_VAR, WARNING_VAR, WHITE_VAR,
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
pub use lightbox::{Lightbox, LightboxHandle, LightboxMedia, lightbox, use_lightbox};
pub use link::{Link, LinkColor, link};
pub use markdown::{ContentAlign, HeadingLinks, Markdown, MarkdownDensity, markdown};
pub use media_overlay::{MediaOverlay, OverlayPosition, OverlayScrim, ShowOn, media_overlay};
pub use media_query::Breakpoint;
pub use mobile_nav::{
    AppShellMobile, DrawerSide, MobileNav, MobileNavToggle, mobile_nav, mobile_nav_toggle,
    use_app_shell_mobile,
};
pub use more_menu::{MoreMenu, more_menu};
pub use nav_heading::{
    NavHeadingMenu, NavHeadingMenuItem, SideNavHeading, nav_heading_menu, nav_heading_menu_item,
    side_nav_heading, top_nav_heading,
};
pub use nav_icon::nav_icon;
pub use outline::{Outline, OutlineDensity, OutlineItem, outline};
pub use pagination::{pagination_count, pagination_page_summary, pagination_status};
pub use power_search::{PowerSearch, SearchFilter, SearchOperator, power_search};
pub use radius::{
    RADIUS_CHAT, RADIUS_CONTAINER, RADIUS_ELEMENT, RADIUS_FULL, RADIUS_INNER, RADIUS_NONE,
    RADIUS_PAGE,
};
pub use rich_text::{RichDoc, RichTextEditor, rich_text_editor};
pub use scrollable_area::{ScrollAxis, ScrollableArea, scrollable_area};
pub use section::{Section, SectionEdges, section};
pub use soft_chat_tools::{ChatToolCalls, ToolCall, ToolStatus, chat_tool_calls};
pub use soft_overflow::{CollapseFrom, OverflowList, OverflowMeasure, overflow_list};
pub use soft_table::{
    RowStatusTone, TableSelectionToolbar, table_row_index, table_row_status,
    table_selection_toolbar,
};
pub use space::{
    SPACE_CARD_PADDING, SPACE_CARD_PADDING_VAR, SPACE_DIALOG_PADDING, SPACE_DIALOG_PADDING_VAR,
    SPACE_LAYER_VIEWPORT_GUTTER, SPACE_LAYER_VIEWPORT_GUTTER_VAR, SPACE_S0, SPACE_S0_5,
    SPACE_S0_5_VAR, SPACE_S0_VAR, SPACE_S1, SPACE_S1_5, SPACE_S1_5_VAR, SPACE_S1_VAR, SPACE_S2,
    SPACE_S2_VAR, SPACE_S3, SPACE_S3_VAR, SPACE_S4, SPACE_S4_VAR, SPACE_S5, SPACE_S5_VAR, SPACE_S6,
    SPACE_S6_VAR, SPACE_S7, SPACE_S7_VAR, SPACE_S8, SPACE_S8_VAR, SPACE_S9, SPACE_S9_VAR,
    SPACE_S10, SPACE_S10_VAR, SPACE_S11, SPACE_S11_VAR, SPACE_S12, SPACE_S12_VAR,
    SPACE_SECTION_PADDING, SPACE_SECTION_PADDING_VAR, Space,
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
pub use thumbnail::{RemoveReveal, Thumbnail, thumbnail};
pub use timer::{Timer, TimerFormat, timer};
pub use timestamp::{
    InstantFormat, TimeZone, Timestamp, TimestampEntry, TimestampFormat, TimestampValue, timestamp,
};
pub use token::{Token, TokenColor, token};
pub use top_nav::{
    TopNav, TopNavFeaturedCard, TopNavItem, TopNavMegaMenu, TopNavMegaMenuItem, TopNavMenu,
    top_nav, top_nav_featured_card, top_nav_item, top_nav_mega_menu, top_nav_mega_menu_item,
    top_nav_menu, top_nav_menu_item,
};
pub use variants::{
    AvatarGroupShape, AvatarSize, BadgeVariant, BannerVariant, BreadcrumbsVariant,
    ButtonGroupVariant, CardPadding, CardVariant, ChatBubbleVariant, ChatDensity,
    CodeBlockContainer, CodeBlockSize, CodeBlockWidth, ComposerElevation, DisclosureChevron,
    DisclosureDensity, DisclosureDividers, DropzoneVariant, Elevation, EmptyStateVariant,
    HeadingDisplay, ItemAlign, ItemDensity, ItemHighlighted, ItemLayout, LabelPosition,
    ListDividers, ListEdge, MetadataColumns, MetadataLabelWidth, MetadataOrientation,
    PaginationSize, RowStatusDot, SectionVariant, SegmentedIconOnly, SegmentedLayout,
    SkeletonRadius, SpinnerShade, SpinnerVariant, StepCollapsedVariant, StepDensity,
    StepIndicatorPosition, SystemMessageVariant, TableCellAlign, TableCellDivider,
    TableRowDividers, TableRowHover, TableRowStriped, TabsDivider, TabsFullBleed, TabsLayout,
    ToolbarVariant, TreeGuides,
};
pub use visually_hidden::{VisuallyHidden, visually_hidden};
