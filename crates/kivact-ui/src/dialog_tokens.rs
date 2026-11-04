//! Role: Passive theme data for dialog.
/// Where a theme centers modal dialogs opened by an attached trigger.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogCenter {
    #[default]
    Anchor,
    Window,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct DialogTokens {
    pub center: DialogCenter,
}
