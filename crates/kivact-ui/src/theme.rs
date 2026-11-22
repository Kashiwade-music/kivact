//! Role: The kit-author contract of the standard components.
//! Recipe tables, recipe states and theme variables.
//!
//! Every standard component has one `<Name>Theme` (a recipe table, a
//! [`ThemeExtension`](kivact_core::style::ThemeExtension)) and the state its
//! recipes read. A kit changes a component's look with
//! `theme.with(ButtonTheme::default().root(|_theme, _look, base| base.border_radius(0)))`;
//! the third argument is the style of the recipe it replaces. To change one
//! part of an installed theme, start from `theme.ext::<XTheme>()` and replace
//! that part; `XTheme::default()` drops the kit's other recipes. A component's
//! values (heights, colors, durations) are theme variables in the module of
//! the component's name.
pub use crate::recipe::Look;
pub use crate::recipe::Recipe;
pub use crate::syntax_colors as syntax;
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
pub use crate::sidebar::look::{SidebarItemState, SidebarState, SidebarTheme};
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

pub mod avatar_group;

pub mod breadcrumb;

pub mod button;

pub mod button_group;

pub mod calendar;

pub mod carousel;

pub mod chat_thread;

pub mod code_block;

pub mod combobox;

pub mod data_table;

pub mod date_input;

pub mod dropzone;

pub mod hover_card;

pub mod item_list;

pub mod navigation_menu;

pub mod pagination;

pub mod skeleton;

pub mod sparkline;

pub mod spinner;

pub mod stepper;

pub mod table;

pub mod time_input;

pub mod toast;

pub mod toggle_group;

pub mod toolbar;

pub mod tree;

pub mod typing_indicator;

/// The theme variables of the `sidebar` component.
pub use crate::sidebar::vars as sidebar;
