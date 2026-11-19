//! Role: Group per-component visual tokens in one theme extension.
use crate::avatar_tokens::AvatarTokens;
use crate::button::look::ButtonTokens;
use crate::data_table_tokens::DataTableTokens;
use crate::dialog::look::DialogTokens;
use crate::disclosure::look::DisclosureTokens;
use crate::progress_tokens::ProgressTokens;
use crate::sparkline_tokens::SparklineTokens;
use crate::toast::look::ToastTokens;
use crate::toggle_button::look::ToggleButtonTokens;
use kivact_core::style::{Theme, ThemeExtension};
#[derive(Clone, Debug, PartialEq, Default)]
pub struct UiTokens {
    pub chat: crate::ChatTokens,
    pub code_block: crate::CodeBlockTokens,
    pub indicators: crate::indicator_values::IndicatorTokens,
    pub tree: crate::collection_values::TreeTokens,
    pub table: crate::collection_values::TableTokens,
    pub item_list: crate::collection_values::ItemListTokens,
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
