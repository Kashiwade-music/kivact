//! Role: Share sidebar nesting and collapsed-rail context.

use kivact::state::Binding;
use kivact::state::Value;
use kivact::state::try_use_context;

#[derive(Clone, Copy, Default)]
pub(super) struct SideNavCx {
    // xtask-allow(state-fields): passive record - collapsed
    pub(super) collapsed: Option<Binding<bool>>,
    pub(super) depth: usize,
    // xtask-allow(state-fields): passive record - popup
    pub(super) popup: Option<Binding<bool>>,
}

pub(super) fn nav_cx() -> SideNavCx {
    try_use_context::<SideNavCx>()
        .map(|value| value.get())
        .unwrap_or_default()
}

impl SideNavCx {
    pub(super) fn rail(self) -> bool {
        self.collapsed.is_some_and(|collapsed| collapsed.get())
    }
}

pub(crate) fn nav_is_rail() -> Value<bool> {
    let cx = nav_cx();
    kivact::state::memo(move || cx.rail())
}
