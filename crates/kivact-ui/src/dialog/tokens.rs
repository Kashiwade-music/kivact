//! Role: Hold the dialog tokens that the theme's `UiTokens` carries.

#[derive(Clone, Debug, PartialEq, Default)]
pub struct DialogTokens {
    pub center: DialogCenter,
}

/// Where a theme centers modal dialogs opened by an attached trigger.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogCenter {
    #[default]
    Anchor,
    Window,
}
