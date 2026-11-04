//! Role: Group per-component visual tokens in one theme extension.
use crate::{
    ButtonTokens, DataTableTokens, DialogTokens, DisclosureTokens, ProgressTokens, ToastTokens,
    ToggleButtonTokens,
};
use kivact_core::style::{Theme, ThemeExtension};
#[derive(Clone, Debug, PartialEq, Default)]
pub struct UiTokens {
    pub button: ButtonTokens,
    pub progress: ProgressTokens,
    pub toast: ToastTokens,
    pub toggle_button: ToggleButtonTokens,
    pub dialog: DialogTokens,
    pub disclosure: DisclosureTokens,
    pub data_table: DataTableTokens,
}
impl ThemeExtension for UiTokens {
    fn fallback(_: &Theme) -> Self {
        Self::default()
    }
}
