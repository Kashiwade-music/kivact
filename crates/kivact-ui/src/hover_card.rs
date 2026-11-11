//! Role: Render a themed delayed preview attached to a caller-owned trigger.
use kivact_core::state::{Binding, IntoBinding, IntoText, IntoValue, Value, effect_on, signal};
use kivact_core::style::use_theme;
use kivact_core::view::{
    Element, HandlerResult, IntoView, OverlayAlign, Placement, Role, View, ViewFn, column,
    node_ref, overlay, show,
};
use kivact_interaction::{HoverCardTiming, InteractionPolicy, use_hover_card};
use std::time::Duration;

use crate::hover_card_values::HoverCardSurfaceState;

/// Whether the trigger participates in focus opening.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FocusTrigger {
    #[default]
    /// Open when the existing trigger or preview gains focus.
    Auto,
    /// Also make the trigger keyboard focusable.
    Always,
    /// Only pointer or explicit opening requests open the preview.
    Never,
}
/// Whether the preview is open, and who decides it.
#[derive(Clone, Copy)]
enum CardOpen {
    /// The caller's binding is the sole authority.
    Controlled(Binding<bool>),
    /// The preview starts at this value and then follows its own behavior.
    Uncontrolled(bool),
}

/// A floating preview with shared pointer corridor, focus, and dismissal behavior.
pub struct HoverCard {
    content: ViewFn,
    variants: crate::variants::Variants,
    placement: Value<Placement>,
    align: Value<OverlayAlign>,
    timing: HoverCardTiming,
    enabled: Value<bool>,
    focus: FocusTrigger,
    open: CardOpen,
    // xtask-allow(state-fields): option: HoverCard::label; absent keeps the preview a Group named by the hover_card.label message, present makes it a named Dialog
    label: Option<Value<String>>,
    on_open_change: Option<kivact_core::state::Callback<bool>>,
}
/// Build the preview content in a fresh owner each time it opens.
pub fn hover_card<V: IntoView>(content: impl Fn() -> V + 'static) -> HoverCard {
    HoverCard {
        content: ViewFn::from(move |()| content()),
        variants: crate::variants::Variants::default(),
        placement: Placement::Below.into_value(),
        align: OverlayAlign::Center.into_value(),
        timing: use_theme().get().ext::<InteractionPolicy>().hover_card,
        enabled: true.into_value(),
        focus: FocusTrigger::Auto,
        open: CardOpen::Uncontrolled(false),
        label: None,
        on_open_change: None,
    }
}
impl HoverCard {
    /// Select a theme-specific visual treatment.
    pub fn variant(mut self, variant: impl crate::Variant) -> Self {
        self.variants.add(variant);
        self
    }

    /// Choose a constant or reactive side of the trigger.
    pub fn placement(mut self, placement: impl IntoValue<Placement>) -> Self {
        self.placement = placement.into_value();
        self
    }
    /// Choose constant or reactive alignment along the trigger.
    pub fn align(mut self, align: impl IntoValue<OverlayAlign>) -> Self {
        self.align = align.into_value();
        self
    }
    /// Override pointer entry delay; focus still opens immediately.
    pub fn open_delay(mut self, delay: Duration) -> Self {
        self.timing.open_delay = delay;
        self
    }
    /// Override the delay after pointer and focus leave.
    pub fn close_delay(mut self, delay: Duration) -> Self {
        self.timing.close_delay = delay;
        self
    }
    /// Use this binding as the sole authority for visibility.
    /// Pointer entry, departure and dismissal request writes; rejected writes
    /// are respected. An external close suppresses the current engagement.
    pub fn open(mut self, open: impl IntoBinding<bool>) -> Self {
        self.open = CardOpen::Controlled(open.into_binding());
        self
    }
    /// Start visible until the first completed visit, unless an open binding is supplied.
    pub fn default_open(mut self, open: bool) -> Self {
        if !matches!(self.open, CardOpen::Controlled(_)) {
            self.open = CardOpen::Uncontrolled(open);
        }
        self
    }
    /// Gate all opening requests with a reactive condition.
    pub fn enabled(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.enabled = enabled.into_value();
        self
    }
    /// Use existing focusability, force a tab stop, or disable focus opening.
    pub fn focus_trigger(mut self, focus: FocusTrigger) -> Self {
        self.focus = focus;
        self
    }
    /// Give interactive content a dialog role and accessible name.
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.label = Some(label.into_text_value());
        self
    }
    /// Observe visibility changes after the initial value.
    pub fn on_open_change<R: HandlerResult>(mut self, f: impl Fn(bool) -> R + 'static) -> Self {
        self.on_open_change = Some(kivact_core::state::callback(f));
        self
    }
    /// Attach without replacing the trigger's role or existing handlers.
    pub fn attach<T: Element + IntoView>(self, trigger: T) -> View {
        let anchor = node_ref();
        let card = node_ref();
        let anchor_focus = anchor.focus_within();
        let card_focus = card.focus_within();
        let focus = self.focus;
        let requested = match self.open {
            CardOpen::Controlled(binding) => binding,
            CardOpen::Uncontrolled(initial) => signal(initial).into_binding(),
        };
        let behavior = use_hover_card(
            anchor,
            card,
            self.enabled,
            move || focus != FocusTrigger::Never && (anchor_focus.get() || card_focus.get()),
            requested,
            self.timing,
        );
        let shown = behavior.is_open();
        if let Some(callback) = self.on_open_change {
            effect_on(shown, move |open| callback.call(*open));
        }
        let trigger = trigger.node_ref(anchor);
        let trigger = if focus == FocusTrigger::Always {
            trigger.focusable(true)
        } else {
            trigger
        };
        let layer = show(shown, move || {
            let part = kivact_core::custom::part(move |input| HoverCardSurfaceState {
                input,
                open: shown.get(),
                placement: self.placement.get(),
            });
            let state = part.state();
            let panel = crate::layer::with_recipe_style(
                column(self.content.call(())).part(part),
                |recipes| recipes.hover_card_recipes().surface_recipe(),
                &self.variants,
                move || state.get(),
            );
            let tokens = use_theme()
                .get()
                .ext::<crate::UiTokens>()
                .hover_card
                .clone();
            let layer = overlay(anchor, self.placement, panel)
                .align(self.align)
                .offset(tokens.offset)
                .viewport_margin(tokens.viewport_margin)
                .node_ref(card)
                .on_dismiss(move |reason| {
                    if reason == kivact_core::view::DismissReason::Escape && card_focus.get() {
                        anchor.focus();
                    }
                    requested.set(false);
                    behavior.hide();
                });
            match self.label {
                Some(label) => layer.with_node(|node| node.role(Role::Dialog)).name(label),
                None => layer
                    .with_node(|node| node.role(Role::Group))
                    .name(kivact_core::i18n::message("hover_card.label", "Preview")),
            }
        });
        (trigger, layer).into_view()
    }
}
