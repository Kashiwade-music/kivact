//! Role: Group per-component visual tokens in one theme extension.

use crate::avatar::look::AvatarTokens;
use crate::button::look::ButtonTokens;
use crate::data_table::look::DataTableTokens;
use crate::dialog::look::DialogTokens;
use crate::disclosure::look::DisclosureTokens;
use crate::progress::look::ProgressTokens;
use crate::sparkline::look::SparklineTokens;
use crate::toast::look::ToastTokens;
use crate::toggle_button::look::ToggleButtonTokens;
use kivact_core::style::{Theme, ThemeExtension};
#[derive(Clone, Debug, PartialEq, Default)]
pub struct UiTokens {
    pub chat: crate::ChatTokens,
    pub code_block: crate::CodeBlockTokens,
    pub indicators: IndicatorTokens,
    pub tree: crate::tree::look::TreeTokens,
    pub table: crate::table::look::TableTokens,
    pub item_list: crate::item_list::look::ItemListTokens,
    pub pagination: crate::NavigationTokens,
    pub breadcrumb: crate::NavigationTokens,
    pub stepper: crate::StepperTokens,
    pub navigation_menu: crate::NavigationTokens,
    pub calendar: crate::CalendarTokens,
    pub date_input: crate::DateInputTokens,
    pub time_input: crate::DateInputTokens,
    pub carousel: crate::CarouselTokens,
    pub dropzone: crate::DropzoneTokens,
    pub combobox: crate::ComboboxTokens,
    pub toolbar: crate::toolbar::look::ToolbarTokens,
    pub toggle_group: crate::toggle_group::look::ToggleGroupTokens,
    pub button_group: crate::button_group::look::ButtonGroupTokens,
    pub avatar: AvatarTokens,
    pub button: ButtonTokens,
    pub progress: ProgressTokens,
    pub sparkline: SparklineTokens,
    pub toast: ToastTokens,
    pub toggle_button: ToggleButtonTokens,
    pub dialog: DialogTokens,
    pub disclosure: DisclosureTokens,
    pub data_table: DataTableTokens,
    pub hover_card: crate::hover_card::look::HoverCardTokens,
}
impl ThemeExtension for UiTokens {
    fn fallback(_: &Theme) -> Self {
        Self::default()
    }
}

/// Common navigation metrics installed in [`crate::UiTokens`].
#[derive(Clone, Debug, PartialEq)]
pub struct NavigationTokens {
    pub gap: f64,
    pub padding: f64,
    pub control_height: f64,
    pub panel_padding: f64,
}

impl Default for NavigationTokens {
    fn default() -> Self {
        Self {
            gap: 8.0,
            padding: 4.0,
            control_height: 32.0,
            panel_padding: 8.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IndicatorTokens {
    pub period_ms: u64,
    /// Essential spinner cycle under reduced motion; absent keeps decorative motion paused.
    pub reduced_period_ms: Option<u64>,
    /// Optional skeleton base tween; index delay and alternating repetition are added by the component.
    pub skeleton_transition: Option<kivact_core::motion::Transition>,
    pub delay_ms: f64,
    pub stagger_ms: f64,
    pub line_height: f64,
    pub gap: f64,
    /// Logical pixels shared by adjacent avatars.
    pub avatar_overlap: f64,
}

impl Default for IndicatorTokens {
    fn default() -> Self {
        Self {
            period_ms: 1000,
            reduced_period_ms: None,
            skeleton_transition: None,
            delay_ms: 0.0,
            stagger_ms: 100.0,
            line_height: 16.0,
            gap: 6.0,
            avatar_overlap: 8.0,
        }
    }
}

/// Shared chat animation metrics.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatTokens {
    pub typing_period_ms: u64,
    /// Distance from the end accepted by the existing follow behavior and jump action.
    pub follow_slack: f64,
}

impl Default for ChatTokens {
    fn default() -> Self {
        Self {
            typing_period_ms: 1000,
            follow_slack: 24.0,
        }
    }
}
