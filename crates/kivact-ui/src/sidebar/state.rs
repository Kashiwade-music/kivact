//! Role: The sidebar's enums, its panel state and the provider's context.
//! Also the menu button's size and the resize width.
//!
//! Parts read the nearest [`SidebarCx`] while they mount, after the provider
//! has established its context boundary. A part used outside a provider
//! behaves as an expanded sidebar, and its trigger does nothing.

use kivact_core::state::{Signal, Value, memo, signal, try_use_context};
use kivact_core::view::Variant;
use kivact_macros::Prop;

/// The width of a resizable sidebar: its inner edge drags it between `min` and `max`.
///
/// `width` is the application's state, so it can persist the width.
#[derive(Clone, Copy, Debug)]
pub struct SidebarResize {
    /// The width in px. Kept between `min` and `max` while the sidebar is open.
    pub width: Signal<f64>,
    /// The narrowest width.
    pub min: f64,
    /// The widest width.
    pub max: f64,
}

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
pub enum SidebarFrame {
    /// A full-height panel with a border on the inner edge.
    #[default]
    Sidebar,
    /// A panel inset by 8 px with an outline and a shadow.
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

/// The height of a [`sidebar_menu_button`](super::sidebar_menu_button) (`size` of sidebar.tsx).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum SidebarMenuButtonSize {
    /// 32 px.
    #[default]
    Default,
    /// 28 px.
    Sm,
    /// 48 px, for rows with a two-line label or an avatar.
    Lg,
}

impl Variant for SidebarMenuButtonSize {}

impl SidebarMenuButtonSize {
    /// The height of the row.
    pub(crate) fn height(self) -> f64 {
        match self {
            SidebarMenuButtonSize::Default => 32.0,
            SidebarMenuButtonSize::Sm => 28.0,
            SidebarMenuButtonSize::Lg => 48.0,
        }
    }

    /// The top of the badge and the action, centered in the row.
    pub(crate) fn corner(self) -> f64 {
        match self {
            SidebarMenuButtonSize::Default => 6.0,
            SidebarMenuButtonSize::Sm => 4.0,
            SidebarMenuButtonSize::Lg => 10.0,
        }
    }
}

/// What the provider shares with the parts below it.
#[derive(Clone, Copy)]
pub(crate) struct SidebarCx {
    pub(crate) open: Signal<bool>,
    pub(crate) side: SidebarSide,
    pub(crate) frame: SidebarFrame,
    pub(crate) collapsible: SidebarCollapsible,
}

/// The resize setting of the provider above the current owner: `None` unless it is resizable.
///
/// It is its own context, so [`SidebarCx`] keeps the open state alone.
#[derive(Clone, Copy)]
pub(crate) struct SidebarResizeCx(pub(crate) Option<SidebarResize>);

impl SidebarCx {
    /// An open sidebar with the defaults, for a sidebar outside a provider.
    pub(crate) fn standalone() -> Self {
        Self {
            open: signal(true),
            side: SidebarSide::Left,
            frame: SidebarFrame::Sidebar,
            collapsible: SidebarCollapsible::Offcanvas,
        }
    }

    /// Open or close the sidebar, unless it cannot close.
    pub(crate) fn toggle(self) {
        if self.collapsible != SidebarCollapsible::None {
            self.open.set(!self.open.get());
        }
    }

    /// Whether the sidebar is currently an icon column.
    pub(crate) fn icon_only(self) -> bool {
        self.collapsible == SidebarCollapsible::Icon && !self.open.get()
    }

    /// The state the panel-level parts read.
    pub(crate) fn state(self) -> SidebarState {
        SidebarState {
            open: self.open.get(),
            icon_only: self.icon_only(),
            side: self.side,
            frame: self.frame,
            collapsible: self.collapsible,
        }
    }
}

/// The sidebar state of the provider above the current owner.
pub(crate) fn current() -> Option<SidebarCx> {
    try_use_context::<SidebarCx>()
}

/// The resize setting of the provider above the current owner.
pub(crate) fn current_resize() -> Option<SidebarResize> {
    try_use_context::<SidebarResizeCx>().and_then(|cx| cx.0)
}

/// The state the parts read: the provider's, or an expanded sidebar's outside one.
pub(crate) fn part_state(cx: Option<SidebarCx>) -> Value<SidebarState> {
    memo(move || {
        cx.map_or(
            SidebarState {
                open: true,
                icon_only: false,
                side: SidebarSide::Left,
                frame: SidebarFrame::Sidebar,
                collapsible: SidebarCollapsible::Offcanvas,
            },
            SidebarCx::state,
        )
    })
}

/// A handle on the sidebar of the provider above the current owner: it reads and toggles the sidebar.
#[derive(Clone, Copy)]
pub struct SidebarHandle {
    cx: SidebarCx,
}

impl SidebarHandle {
    /// Whether the sidebar is an icon column: it is closed and collapses to icons.
    pub fn icon_only(self) -> bool {
        self.cx.icon_only()
    }

    /// The state the sidebar's parts read.
    pub fn state(self) -> SidebarState {
        self.cx.state()
    }

    /// Open or close the sidebar, unless it cannot close.
    pub fn toggle(self) {
        self.cx.toggle();
    }
}

/// The handle on the sidebar of the provider above the current owner, or `None` outside a provider.
pub fn use_sidebar() -> Option<SidebarHandle> {
    current().map(|cx| SidebarHandle { cx })
}

/// What the panel-level parts of a sidebar read.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SidebarState {
    /// Whether the sidebar is open.
    pub open: bool,
    /// Whether the sidebar is an icon column (`Icon` and closed).
    pub icon_only: bool,
    /// The side of the window.
    pub side: SidebarSide,
    /// The frame.
    pub frame: SidebarFrame,
    /// What closing does.
    pub collapsible: SidebarCollapsible,
}

/// What a row of a sidebar menu (and its badge, action and sub list) reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SidebarItemState {
    /// Whether the row is the current page.
    pub active: bool,
    /// Whether the sidebar is an icon column.
    pub icon_only: bool,
    /// Whether the row has an action control.
    pub has_action: bool,
    /// Whether the reading direction is right to left.
    pub rtl: bool,
    /// Whether the row is disabled: it takes no press.
    pub disabled: bool,
    /// The row's height.
    pub size: SidebarMenuButtonSize,
    /// Whether the row's sub list is open, or `None` when the row has no toggle.
    pub expanded: Option<bool>,
}
