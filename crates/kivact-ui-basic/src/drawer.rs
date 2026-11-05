//! Role: Basic visual variants for standard edge dialogs.
/// The visual treatment of an edge dialog.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DrawerVariant {
    /// A swipe drawer with its visible grab handle.
    #[default]
    Drawer,
    /// A sheet without the visible grab handle.
    Sheet,
}
impl kivact::ui::Variant for DrawerVariant {}
