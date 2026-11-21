//! Role: Bind domain navigation surfaces to one canonical exclusive owner.
use kivact::interaction::{Delays, NavigationMenuApi, NavigationPanelApi, use_navigation_menu};
use kivact::state::{
    IntoValue, Signal, Value, binding, effect_on, on_cleanup, provide, signal, try_use_context,
};
use kivact::view::{Element, Handled, IntoView, PointerEvent, PointerKind, PointerPhase, View};
use std::time::Duration;
#[derive(Clone, Copy)]
struct NavigationGroup(NavigationMenuApi);
/// Establish one owner around actual navigation content, without adding a view node.
pub(crate) fn with_navigation_group<V: IntoView>(content: impl FnOnce() -> V + 'static) -> View {
    provide(NavigationGroup(use_navigation_menu(signal(false))), content)
}
#[derive(Clone, Copy)]
pub(crate) struct NavPanel {
    panel: NavigationPanelApi,
    open: Signal<bool>,
    eligible: Value<bool>,
    delays: Delays,
}
impl NavPanel {
    pub(crate) fn new(eligible: impl IntoValue<bool>, delays: Delays) -> Self {
        let api = try_use_context::<NavigationGroup>()
            .map_or_else(|| use_navigation_menu(signal(false)), |group| group.0);
        let panel = api.panel();
        on_cleanup(move || panel.removed());
        let eligible = eligible.into_value();
        let active = move || panel.is_open().get();
        let open = binding(
            move || eligible.get() && active(),
            move |open| {
                if open && eligible.get_untracked() {
                    panel.show();
                } else if !open && active() {
                    api.hide();
                }
            },
        );
        // Independent eligibility changes send real controller events, never reconcile an open copy.
        effect_on(eligible, move |eligible| {
            if !eligible {
                panel.removed();
            }
        });
        Self {
            panel,
            open,
            eligible,
            delays,
        }
    }
    pub(crate) fn open(self) -> Signal<bool> {
        self.open
    }
    pub(crate) fn toggle(self) {
        if self.eligible.get_untracked() {
            self.panel.toggle();
        }
    }
    pub(crate) fn trigger<T: Element>(self, target: T) -> T {
        target.with_node(move |node| {
            node.on_pointer(move |event| self.pointer(event, NavPart::Trigger))
        })
    }
    pub(crate) fn surface<T: Element>(self, target: T) -> T {
        target.with_node(move |node| {
            node.on_pointer(move |event| self.pointer(event, NavPart::Surface))
        })
    }
    fn pointer(self, event: PointerEvent, part: NavPart) -> Handled {
        match event.phase() {
            PointerPhase::Enter
                if self.eligible.get_untracked() && event.kind() != PointerKind::Touch =>
            {
                self.panel.hover_after(if matches!(part, NavPart::Surface) {
                    Duration::ZERO
                } else {
                    self.delays.open
                })
            }
            PointerPhase::Leave => self.panel.leave_after(self.delays.close),
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
