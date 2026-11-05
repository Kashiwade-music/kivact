//! Role: Group per-component visual tokens in one theme extension.
use crate::button_tokens::ButtonTokens;
use crate::data_table_tokens::DataTableTokens;
use crate::dialog_tokens::DialogTokens;
use crate::disclosure_tokens::DisclosureTokens;
use crate::progress_tokens::ProgressTokens;
use crate::toast_tokens::ToastTokens;
use crate::toggle_button_tokens::ToggleButtonTokens;
use kivact_core::style::{Theme, ThemeExtension};
#[derive(Clone, Debug, PartialEq, Default)]
pub struct UiTokens {
    pub indicators: crate::IndicatorTokens,
    pub tree: crate::TreeTokens,
    pub table: crate::TableTokens,
    pub item_list: crate::ItemListTokens,
    pub pagination: crate::NavigationTokens,
    pub breadcrumb: crate::NavigationTokens,
    pub stepper: crate::NavigationTokens,
    pub navigation_menu: crate::NavigationTokens,
    pub calendar: crate::CalendarTokens,
    pub date_input: crate::DateInputTokens,
    pub time_input: crate::DateInputTokens,
    pub carousel: crate::CarouselTokens,
    pub dropzone: crate::DropzoneTokens,
    pub combobox: crate::ComboboxTokens,
    pub toolbar: crate::ToolbarTokens,
    pub toggle_group: crate::ToggleGroupTokens,
    pub button_group: crate::ButtonGroupTokens,
    pub button: ButtonTokens,
    pub progress: ProgressTokens,
    pub toast: ToastTokens,
    pub toggle_button: ToggleButtonTokens,
    pub dialog: DialogTokens,
    pub disclosure: DisclosureTokens,
    pub data_table: DataTableTokens,
    pub hover_card: crate::HoverCardTokens,
}
impl ThemeExtension for UiTokens {
    fn fallback(_: &Theme) -> Self {
        Self::default()
    }
}
