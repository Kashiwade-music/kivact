//! Role: Choose touch surfaces for keyed selection components.
/// Presentation used when a touch gesture opens an option list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectPresentation {
    /// Always use an anchored popup.
    #[default]
    Popover,
    /// Use a bottom drawer for touch activation.
    BottomSheet,
    /// Use a bottom drawer when the host supports touch input.
    Adaptive,
}
impl SelectPresentation {
    pub(crate) fn uses_sheet(self) -> bool {
        match self {
            Self::Popover => false,
            Self::BottomSheet => true,
            Self::Adaptive => kivact_core::platform::capabilities().get().touch,
        }
    }
}
