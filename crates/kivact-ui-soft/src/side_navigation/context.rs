//! Role: Share sidebar nesting and collapsed-rail context.

use kivact::state::Binding;
use kivact::state::Value;
use kivact::state::try_use_context;

/// How the surrounding navigation surface shows its labels.
#[derive(Clone, Copy, Default)]
pub(super) enum NavSurface {
    /// Labels and children are always shown.
    #[default]
    Fixed,
    /// The rail follows the collapse binding; its flyouts hold nested items.
    Collapsible(Binding<bool>),
    /// The surface is a rail flyout, so labels and children are shown.
    Flyout {
        /// The flyout's open state.
        open: Binding<bool>,
    },
}

#[derive(Clone, Copy, Default)]
pub(super) struct SideNavCx {
    pub(super) surface: NavSurface,
    pub(super) depth: usize,
}

pub(super) fn nav_cx() -> SideNavCx {
    try_use_context::<SideNavCx>()
        .map(|value| value.get())
        .unwrap_or_default()
}

impl SideNavCx {
    pub(super) fn rail(self) -> bool {
        match self.surface {
            NavSurface::Fixed | NavSurface::Flyout { .. } => false,
            NavSurface::Collapsible(collapsed) => collapsed.get(),
        }
    }

    /// The collapse binding of a collapsible sidebar.
    pub(super) fn collapsed(self) -> Option<Binding<bool>> {
        match self.surface {
            NavSurface::Collapsible(collapsed) => Some(collapsed),
            NavSurface::Fixed | NavSurface::Flyout { .. } => None,
        }
    }

    /// The flyout's open binding.
    pub(super) fn popup(self) -> Option<Binding<bool>> {
        match self.surface {
            NavSurface::Flyout { open } => Some(open),
            NavSurface::Fixed | NavSurface::Collapsible(_) => None,
        }
    }
}

pub(crate) fn nav_is_rail() -> Value<bool> {
    let cx = nav_cx();
    kivact::state::memo(move || cx.rail())
}
