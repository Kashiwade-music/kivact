//! Role: Bind domain navigation surfaces to one canonical exclusive owner.
use kivact::interaction::{Delays, NavOpen, NavigationMenuApi, use_navigation_menu};
use kivact::state::{
    Binding, IntoValue, Value, binding, effect_on, on_cleanup, provide, try_use_context,
};
use kivact::view::{Handled, IntoView, NodeBuilder, PointerEvent, PointerPhase, View};
use std::time::Duration;
#[derive(Clone, Copy)]
struct NavigationGroup(NavigationMenuApi);
/// Establish one owner around actual navigation content, without adding a view node.
pub(crate) fn with_navigation_group<V: IntoView>(content: impl FnOnce() -> V + 'static) -> View {
    provide(NavigationGroup(use_navigation_menu()), content)
}
// xtask-allow(state-fields): Immutable actual panel identity, accepted binding and independent eligibility/delay inputs; NavOpen is the sole lifecycle state.
#[derive(Clone, Copy)]
pub(crate) struct NavPanel {
    api: NavigationMenuApi,
    key: usize,
    open: Binding<bool>,
    eligible: Value<bool>,
    delays: Delays,
}
impl NavPanel {
    pub(crate) fn new(eligible: impl IntoValue<bool>, delays: Delays) -> Self {
        let api = try_use_context::<NavigationGroup>()
            .map_or_else(use_navigation_menu, |group| group.get().0);
        let key = api.panel_key();
        let state = api.state();
        on_cleanup(move || api.panel_removed(key));
        let eligible = eligible.into_value();
        let active =
            move || matches!(state.get(),NavOpen::Hover(at)|NavOpen::Pinned(at) if at==key);
        let open = binding(
            move || eligible.get() && active(),
            move |open| {
                if open && eligible.get_untracked() {
                    api.pin(key);
                } else if !open && active() {
                    api.close();
                }
            },
        );
        // Independent eligibility changes send real controller events, never reconcile an open copy.
        effect_on(eligible, move |eligible| {
            if !eligible {
                api.panel_removed(key);
            }
        });
        Self {
            api,
            key,
            open,
            eligible,
            delays,
        }
    }
    pub(crate) fn open(self) -> Binding<bool> {
        self.open
    }
    pub(crate) fn toggle(self) {
        if self.eligible.get_untracked() {
            self.api.toggle(self.key);
        }
    }
    pub(crate) fn trigger<T: NodeBuilder>(self, target: T) -> T {
        target.with_node(move |node| {
            node.on_pointer(move |event| self.pointer(event, NavPart::Trigger))
        })
    }
    pub(crate) fn surface<T: NodeBuilder>(self, target: T) -> T {
        target.with_node(move |node| {
            node.on_pointer(move |event| self.pointer(event, NavPart::Surface))
        })
    }
    fn pointer(self, event: PointerEvent, part: NavPart) -> Handled {
        match event.phase() {
            PointerPhase::Enter
                if self.eligible.get_untracked()
                    && !kivact::platform::capabilities().get_untracked().touch =>
            {
                self.api.hover_after(
                    self.key,
                    if matches!(part, NavPart::Surface) {
                        Duration::ZERO
                    } else {
                        self.delays.open
                    },
                )
            }
            PointerPhase::Leave => self.api.leave_after(self.key, self.delays.close),
            _ => (),
        }
        Handled::No
    }
}

#[derive(Clone, Copy)]
enum NavPart {
    Trigger,
    Surface,
}
