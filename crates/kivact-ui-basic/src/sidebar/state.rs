//! Role: The sidebar's shared enums and the context the provider gives its parts.
//!
//! The parts of a sidebar (menu buttons, group labels, the trigger) are built
//! before the provider that contains them, so they cannot read the provider's
//! context while they are built. Each part that depends on the sidebar's state
//! therefore builds itself when it is mounted, below the provider, through
//! [`deferred`]. A part used outside any provider behaves as in an expanded
//! sidebar and its trigger does nothing.

use kivact::view::{IntoView, View, provide, try_context};
use kivact::{Binding, Value};

/// The side of the window a sidebar sits on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SidebarSide {
    /// At the left edge.
    #[default]
    Left,
    /// At the right edge.
    Right,
}

/// How a sidebar is framed (`variant` of sidebar.tsx).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SidebarVariant {
    /// A full-height panel with a border on the inner edge.
    #[default]
    Sidebar,
    /// A panel inset by 8 px with a ring and a shadow.
    Floating,
    /// A panel inset by 8 px on the sidebar color, with the page as a card
    /// beside it.
    Inset,
}

/// What a closed sidebar does (`collapsible` of sidebar.tsx).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SidebarCollapsible {
    /// It slides out of view completely.
    #[default]
    Offcanvas,
    /// It shrinks to a column of icons (48 px) with tooltips.
    Icon,
    /// It cannot be closed.
    None,
}

/// The width of an open sidebar.
pub(crate) const WIDTH: f64 = 256.0;
/// The width of an icon-collapsed sidebar.
pub(crate) const WIDTH_ICON: f64 = 48.0;

/// What the provider shares with the parts below it.
#[derive(Clone, Copy)]
pub(crate) struct SidebarCx {
    pub(crate) open: Binding<bool>,
    pub(crate) shown: Value<bool>,
    pub(crate) side: SidebarSide,
    pub(crate) variant: SidebarVariant,
    pub(crate) collapsible: SidebarCollapsible,
}

impl SidebarCx {
    /// Open or close the sidebar, unless it cannot close.
    pub(crate) fn toggle(self) {
        if self.collapsible != SidebarCollapsible::None {
            self.open.set(!self.shown.get());
        }
    }

    /// Whether the sidebar is currently an icon column.
    pub(crate) fn icon_only(self) -> bool {
        self.collapsible == SidebarCollapsible::Icon && !self.shown.get()
    }
}

/// The sidebar state of the provider above the current owner.
pub(crate) fn current() -> Option<SidebarCx> {
    try_context::<SidebarCx>()
}

/// Build `build` when it is mounted, below the provider that contains it.
///
/// Parts are created before the provider runs; this is the only way for
/// them to see its context.
#[track_caller]
pub(crate) fn deferred<V: IntoView + 'static>(build: impl FnOnce() -> V + 'static) -> View {
    provide((), build)
}
