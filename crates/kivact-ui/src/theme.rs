//! Role: The kit-author contract of the standard components: recipe tables, recipe states and theme variables.
//!
//! Every standard component has one `<Name>Theme` (a recipe table, a
//! [`ThemeExtension`](kivact_core::style::ThemeExtension)) and the state its
//! recipes read. A kit changes a component's look with
//! `theme.with(ButtonTheme::default().root(|_theme, _look, base| base.border_radius(0)))`;
//! the third argument is the style of the recipe it replaces. A component's
//! values (heights, colors, durations) are theme variables in the module of
//! the component's name.
pub use crate::recipe::Look;
pub use crate::recipe::Recipe;
pub use kivact_core::view::Variant;

pub use crate::alert::look::AlertState;
pub use crate::alert::look::AlertTheme;
pub use crate::alert::look::Expansion;
pub use crate::announcer_outlet::look::AnnouncerOutletTheme;
pub use crate::async_view::look::AsyncViewTheme;
pub use crate::avatar::look::AvatarState;
pub use crate::avatar::look::AvatarTheme;
pub use crate::avatar_group::look::AvatarGroupTheme;
pub use crate::badge::look::BadgeState;
pub use crate::badge::look::BadgeTheme;
pub use crate::breadcrumb::look::BreadcrumbItemState;
pub use crate::breadcrumb::look::BreadcrumbTheme;
pub use crate::button::look::ButtonState;
pub use crate::button::look::ButtonTheme;
pub use crate::button_group::look::ButtonGroupTheme;
pub use crate::button_group::look::{ButtonGroupMemberState, ButtonGroupState};
pub use crate::calendar::look::CalendarDayVisual;
pub use crate::calendar::look::CalendarEventVisual;
pub use crate::calendar::look::CalendarTheme;
pub use crate::card::look::CardState;
pub use crate::card::look::CardTheme;
pub use crate::card::look::Selectable;
pub use crate::carousel::look::CarouselSurfaceState;
pub use crate::carousel::look::CarouselTheme;
pub use crate::chat_composer::look::ChatComposerState;
pub use crate::chat_composer::look::ChatComposerTheme;
pub use crate::chat_message::look::ChatMessageState;
pub use crate::chat_message::look::ChatMessageTheme;
pub use crate::chat_thread::look::ChatThreadState;
pub use crate::chat_thread::look::ChatThreadTheme;
pub use crate::code_block::look::CodeBlockState;
pub use crate::code_block::look::CodeBlockTheme;
pub use crate::combobox::look::ComboboxTheme;
pub use crate::command_palette::look::CommandPaletteTheme;
pub use crate::data_table::look::DataTableCellState;
pub use crate::data_table::look::DataTableCheckState;
pub use crate::data_table::look::DataTableGroupState;
pub use crate::data_table::look::DataTablePageState;
pub use crate::data_table::look::DataTableRowState;
pub use crate::data_table::look::DataTableTheme;
pub use crate::date_input::look::DateInputTheme;
pub use crate::date_range_input::look::DateRangeInputTheme;
pub use crate::date_time_input::look::DateTimeInputTheme;
pub use crate::description_list::look::DescriptionListTheme;
pub use crate::dialog::look::DialogState;
pub use crate::dialog::look::DialogTheme;
pub use crate::disclosure::look::DisclosureGroupPosition;
pub use crate::disclosure::look::DisclosureRecipeState;
pub use crate::disclosure::look::DisclosureTheme;
pub use crate::dropzone::look::DropzoneTheme;
pub use crate::empty_state::look::EmptyStateState;
pub use crate::empty_state::look::EmptyStateTheme;
pub use crate::field::look::FieldState;
pub use crate::field::look::FieldTheme;
pub use crate::heading::look::HeadingState;
pub use crate::heading::look::HeadingTheme;
pub use crate::hover_card::look::HoverCardSurfaceState;
pub use crate::hover_card::look::HoverCardTheme;
pub use crate::input_group::look::InputGroupMemberKind;
pub use crate::input_group::look::InputGroupMemberState;
pub use crate::input_group::look::InputGroupState;
pub use crate::input_group::look::InputGroupTheme;
pub use crate::item_list::look::ItemListTheme;
pub use crate::item_list::look::ItemRowState;
pub use crate::kbd::look::KbdKeyState;
pub use crate::kbd::look::KbdTheme;
pub use crate::list::look::ListTheme;
pub use crate::menu::look::MenuTheme;
pub use crate::navigation_menu::look::NavigationLinkState;
pub use crate::navigation_menu::look::NavigationMenuTheme;
pub use crate::navigation_menu::look::NavigationTriggerState;
pub use crate::number_input::look::NumberInputTheme;
pub use crate::pagination::look::PaginationItemState;
pub use crate::pagination::look::PaginationSummary;
pub use crate::pagination::look::PaginationTheme;
pub use crate::progress::look::ProgressState;
pub use crate::progress::look::ProgressTheme;
pub use crate::radio_group::look::RadioGroupTheme;
pub use crate::select::look::SelectTheme;
pub use crate::select::look::SelectTriggerState;
pub use crate::skeleton::look::SkeletonState;
pub use crate::skeleton::look::SkeletonTheme;
pub use crate::slider::look::AxisState;
pub use crate::slider::look::SliderTheme;
pub use crate::sparkline::look::SparklineState;
pub use crate::sparkline::look::SparklineTheme;
pub use crate::sparkline::look::sparkline_line;
pub use crate::spinner::look::SpinnerState;
pub use crate::spinner::look::SpinnerTheme;
pub use crate::split::look::SplitTheme;
pub use crate::splitter::look::SplitterTheme;
pub use crate::stat::look::StatState;
pub use crate::stat::look::StatTheme;
pub use crate::stepper::look::StepperItemState;
pub use crate::stepper::look::StepperState;
pub use crate::stepper::look::StepperTheme;
pub use crate::table::look::TableTheme;
pub use crate::tabs::look::TabsTheme;
pub use crate::tag_input::look::TagInputTheme;
pub use crate::text_input::look::TextInputState;
pub use crate::text_input::look::{TextAreaTheme, TextInputTheme};
pub use crate::time_input::look::TimeInputTheme;
pub use crate::toast::look::ToastGroupState;
pub use crate::toast::look::ToastTheme;
pub use crate::toggle::look::{CheckControl, CheckableState};
pub use crate::toggle::look::{CheckboxTheme, SwitchTheme};
pub use crate::toggle_button::look::ToggleButtonState;
pub use crate::toggle_button::look::ToggleButtonTheme;
pub use crate::toggle_group::look::ToggleGroupTheme;
pub use crate::toggle_group::look::{ToggleGroupItemState, ToggleGroupState};
pub use crate::toolbar::look::ToolbarState;
pub use crate::toolbar::look::ToolbarTheme;
pub use crate::tooltip::look::TooltipArrowState;
pub use crate::tooltip::look::TooltipTheme;
pub use crate::tree::look::TreeRowRecipeState;
pub use crate::tree::look::TreeTheme;
pub use crate::typing_indicator::look::TypingIndicatorState;
pub use crate::typing_indicator::look::TypingIndicatorTheme;

pub use crate::dialog::look::DialogCenter;
pub use crate::disclosure::look::DisclosureMotion;
pub use crate::progress::look::ProgressMotion;
pub use crate::sparkline::look::SparklineFill;
pub use crate::toggle_button::look::ToggleLabelReservation;

/// The theme variables of the `avatar_group` component.
pub mod avatar_group {
    pub use crate::avatar_group::look::{OVERLAP, OVERLAP_VAR};
}

/// The theme variables of the `breadcrumb` component.
pub mod breadcrumb {
    pub use crate::breadcrumb::look::{
        CONTROL_HEIGHT, CONTROL_HEIGHT_VAR, GAP, GAP_VAR, PADDING, PADDING_VAR, PANEL_PADDING,
        PANEL_PADDING_VAR,
    };
}

/// The theme variables of the `button` component.
pub mod button {
    pub use crate::button::look::{
        BUSY_REDUCED_SPIN_MS_VAR, BUSY_REVEAL_DELAY_MS_VAR, BUSY_REVEAL_FADE_MS_VAR,
        BUSY_SPIN_MS_VAR,
    };
}

/// The theme variables of the `button_group` component.
pub mod button_group {
    pub use crate::button_group::look::{
        DIVIDER_VAR, GAP, GAP_VAR, RADIUS, RADIUS_VAR, SOLID_DIVIDER_VAR, TEXT_BACKGROUND_VAR,
        TEXT_COLOR_VAR,
    };
}

/// The theme variables of the `calendar` component.
pub mod calendar {
    pub use crate::calendar::look::{
        CELL_SIZE, CELL_SIZE_VAR, MONTH_GAP, MONTH_GAP_VAR, WEEK_GAP, WEEK_GAP_VAR,
    };
}

/// The theme variables of the `carousel` component.
pub mod carousel {
    pub use crate::carousel::look::{
        EDGE_FADE_WIDTH, EDGE_FADE_WIDTH_VAR, GAP, GAP_VAR, PADDING, PADDING_VAR,
    };
}

/// The theme variables of the `chat_thread` component.
pub mod chat_thread {
    pub use crate::chat_thread::look::{FOLLOW_SLACK, FOLLOW_SLACK_VAR};
}

/// The theme variables of the `code_block` component.
pub mod code_block {
    pub use crate::code_block::look::{
        COPY_FEEDBACK_MS_VAR, FONT_SIZE, FONT_SIZE_VAR, LINE_HEIGHT, LINE_HEIGHT_VAR,
    };
}

/// The theme variables of the `combobox` component.
pub mod combobox {
    pub use crate::combobox::look::{
        ACTIVE_BACKGROUND, ACTIVE_BACKGROUND_VAR, BACKGROUND, BACKGROUND_VAR, BORDER, BORDER_VAR,
        CHIP_BACKGROUND, CHIP_BACKGROUND_VAR, CHIP_HEIGHT, CHIP_HEIGHT_VAR, CHIP_RADIUS,
        CHIP_RADIUS_VAR, DISABLED_OPACITY_VAR, EMPTY_FONT_SIZE, EMPTY_FONT_SIZE_VAR, FIELD_HEIGHT,
        FIELD_HEIGHT_VAR, FIELD_PADDING, FIELD_PADDING_VAR, FIELD_RADIUS, FIELD_RADIUS_VAR, FOCUS,
        FOCUS_RING, FOCUS_RING_VAR, FOCUS_VAR, FOREGROUND, FOREGROUND_VAR, LIST_HEIGHT,
        LIST_HEIGHT_VAR, MUTED, MUTED_VAR, OPTION_HEIGHT, OPTION_HEIGHT_VAR, OPTION_PADDING_X,
        OPTION_PADDING_X_VAR, OPTION_PADDING_Y, OPTION_PADDING_Y_VAR, OPTION_RADIUS,
        OPTION_RADIUS_VAR, POPUP_OFFSET, POPUP_OFFSET_VAR,
    };
}

/// The theme variables of the `data_table` component.
pub mod data_table {
    pub use crate::data_table::look::{
        ROW_STATUS_ACCENT, ROW_STATUS_ACCENT_VAR, ROW_STATUS_DANGER, ROW_STATUS_DANGER_VAR,
        ROW_STATUS_INFO, ROW_STATUS_INFO_VAR, ROW_STATUS_NEUTRAL, ROW_STATUS_NEUTRAL_VAR,
        ROW_STATUS_SUCCESS, ROW_STATUS_SUCCESS_VAR, ROW_STATUS_WARNING, ROW_STATUS_WARNING_VAR,
        STICKY_HOVER, STICKY_HOVER_VAR, STICKY_SELECTED, STICKY_SELECTED_VAR, STICKY_SHADOW,
        STICKY_SHADOW_VAR, STICKY_SHADOW_WIDTH, STICKY_SHADOW_WIDTH_VAR, STICKY_SURFACE,
        STICKY_SURFACE_VAR,
    };
}

/// The theme variables of the `date_input` component.
pub mod date_input {
    pub use crate::date_input::look::{
        WHEEL_ROW_HEIGHT, WHEEL_ROW_HEIGHT_VAR, WHEEL_WIDTH, WHEEL_WIDTH_VAR,
    };
}

/// The theme variables of the `dropzone` component.
pub mod dropzone {
    pub use crate::dropzone::look::{MIN_HEIGHT, MIN_HEIGHT_VAR, PADDING, PADDING_VAR};
}

/// The theme variables of the `hover_card` component.
pub mod hover_card {
    pub use crate::hover_card::look::{OFFSET, OFFSET_VAR, VIEWPORT_MARGIN, VIEWPORT_MARGIN_VAR};
}

/// The theme variables of the `item_list` component.
pub mod item_list {
    pub use crate::item_list::look::{
        DESCRIPTION_SIZE, DESCRIPTION_SIZE_VAR, GAP, GAP_VAR, PADDING, PADDING_VAR, ROW_GAP,
        ROW_GAP_VAR,
    };
}

/// The theme variables of the `navigation_menu` component.
pub mod navigation_menu {
    pub use crate::navigation_menu::look::{
        CONTROL_HEIGHT, CONTROL_HEIGHT_VAR, GAP, GAP_VAR, PADDING, PADDING_VAR, PANEL_PADDING,
        PANEL_PADDING_VAR,
    };
}

/// The theme variables of the `pagination` component.
pub mod pagination {
    pub use crate::pagination::look::{
        CONTROL_HEIGHT, CONTROL_HEIGHT_VAR, GAP, GAP_VAR, PADDING, PADDING_VAR, PANEL_PADDING,
        PANEL_PADDING_VAR,
    };
}

/// The theme variables of the `skeleton` component.
pub mod skeleton {
    pub use crate::skeleton::look::{
        DELAY_MS, DELAY_MS_VAR, GAP, GAP_VAR, LINE_HEIGHT, LINE_HEIGHT_VAR, PERIOD_MS,
        PERIOD_MS_VAR, STAGGER_MS, STAGGER_MS_VAR,
    };
}

/// The theme variables of the `sparkline` component.
pub mod sparkline {
    pub use crate::sparkline::look::{
        INSET, INSET_VAR, LINE, LINE_VAR, LINE_WIDTH, LINE_WIDTH_VAR,
    };
}

/// The theme variables of the `spinner` component.
pub mod spinner {
    pub use crate::spinner::look::{GAP, GAP_VAR, PERIOD_MS, PERIOD_MS_VAR};
}

/// The theme variables of the `stepper` component.
pub mod stepper {
    pub use crate::stepper::look::{
        CONNECTOR_GAP, CONNECTOR_GAP_VAR, GAP, GAP_VAR, MIN_STEP_WIDTH, MIN_STEP_WIDTH_VAR,
        PADDING, PADDING_VAR,
    };
}

/// The theme variables of the `table` component.
pub mod table {
    pub use crate::table::look::{
        CELL_PADDING, CELL_PADDING_VAR, FONT_SIZE, FONT_SIZE_VAR, HEADER_HEIGHT, HEADER_HEIGHT_VAR,
    };
}

/// The theme variables of the `time_input` component.
pub mod time_input {
    pub use crate::time_input::look::{
        WHEEL_ROW_HEIGHT, WHEEL_ROW_HEIGHT_VAR, WHEEL_WIDTH, WHEEL_WIDTH_VAR,
    };
}

/// The theme variables of the `toast` component.
pub mod toast {
    pub use crate::toast::look::{ROW_GAP, ROW_GAP_VAR};
}

/// The theme variables of the `toggle_group` component.
pub mod toggle_group {
    pub use crate::toggle_group::look::{GAP, GAP_VAR};
}

/// The theme variables of the `toolbar` component.
pub mod toolbar {
    pub use crate::toolbar::look::{
        BACKGROUND, BACKGROUND_VAR, BORDER, BORDER_VAR, GAP, GAP_VAR, MIN_HEIGHT, MIN_HEIGHT_VAR,
        PADDING, PADDING_VAR,
    };
}

/// The theme variables of the `tree` component.
pub mod tree {
    pub use crate::tree::look::{
        GAP, GAP_VAR, INDENT, INDENT_VAR, MARKER_WIDTH, MARKER_WIDTH_VAR, PADDING, PADDING_VAR,
        RADIUS, RADIUS_VAR, ROW_HEIGHT, ROW_HEIGHT_VAR,
    };
}

/// The theme variables of the `typing_indicator` component.
pub mod typing_indicator {
    pub use crate::typing_indicator::look::{PERIOD_MS, PERIOD_MS_VAR};
}
