//! Role: Present persistent alerts and dismissible feedback banners.
use super::layer::{with_base_recipe_style, with_recipe_style};
use super::variants::{Variant, Variants};
use kivact_core::custom::{NodeToken, Owner};
use kivact_core::state::{
    Binding, Callback, IntoBinding, IntoText, IntoValue, Machine, MachineHandle, Step, Value,
    effect, machine, signal,
};
use kivact_core::view::{
    IntoView, IntoViewFn, LiveRegion, Node, NodeBuilder, NodeRef, Role, View, ViewFn, node,
    node_ref, show, text,
};
use kivact_interaction::Tone;

#[derive(Clone, Copy, PartialEq)]
enum LocalBannerState {
    Shown { restore: Option<NodeToken> },
    Dismissed,
}
#[derive(Clone, Copy)]
enum BannerState {
    Local(LocalBannerState),
    Shared {
        visible: Binding<bool>,
        restore: Option<NodeToken>,
    },
}
impl PartialEq for BannerState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Local(a), Self::Local(b)) => a == b,
            (
                Self::Shared {
                    visible: a,
                    restore: ar,
                },
                Self::Shared {
                    visible: b,
                    restore: br,
                },
            ) => a.into_value() == b.into_value() && ar == br,
            _ => false,
        }
    }
}
impl BannerState {
    fn visible(self) -> bool {
        match self {
            Self::Local(LocalBannerState::Shown { .. }) => true,
            Self::Local(LocalBannerState::Dismissed) => false,
            Self::Shared { visible, .. } => visible.get(),
        }
    }
}
use crate::component_values::AlertState;

/// An urgent, persistent message with optional actions and details.
pub struct Alert {
    root: Node,
    title: Value<String>,
    // xtask-allow(state-fields): builder option - description
    description: Option<Value<String>>,
    // xtask-allow(state-fields): builder option - icon
    icon: Option<View>,
    // xtask-allow(state-fields): builder option - actions
    actions: Option<View>,
    content: Option<ViewFn>,
    collapsible: Value<bool>,
    // xtask-allow(state-fields): builder option - open
    open: Option<Binding<bool>>,
    tone: Value<Tone>,
    dismissible: Value<bool>,
    dismiss_label: Value<String>,
    // xtask-allow(state-fields): builder option - dismiss content
    dismiss_content: Option<View>,
    dismiss: Option<Callback<()>>,
    // xtask-allow(state-fields): builder option - visible
    visible: Option<Binding<bool>>,
    variants: Variants,
}
/// Display an alert without moving keyboard focus or automatically hiding it.
pub fn alert(title: impl IntoText) -> Alert {
    let title = title.into_text_value();
    Alert {
        root: node()
            .role(Role::Alert)
            .live_region(LiveRegion::Assertive)
            .name(title),
        title,
        description: None,
        icon: None,
        actions: None,
        content: None,
        collapsible: false.into_value(),
        open: None,
        tone: Tone::Neutral.into_value(),
        dismissible: false.into_value(),
        dismiss_label: "Dismiss notification".into_text_value(),
        dismiss_content: None,
        dismiss: None,
        visible: None,
        variants: Variants::default(),
    }
}
impl Alert {
    /// Set supporting text.
    pub fn description(mut self, value: impl IntoText) -> Self {
        self.description = Some(value.into_text_value());
        self
    }
    /// Set the semantic tone.
    pub fn tone(mut self, tone: impl IntoValue<Tone>) -> Self {
        self.tone = tone.into_value();
        self
    }
    /// Set the decorative icon.
    pub fn icon(mut self, value: impl IntoView) -> Self {
        self.icon = Some(value.into_view());
        self
    }
    /// Set trailing actions.
    pub fn actions(mut self, value: impl IntoView) -> Self {
        self.actions = Some(value.into_view());
        self
    }
    /// Build details each time they are expanded.
    pub fn content<V: IntoView>(mut self, content: impl IntoViewFn<V>) -> Self {
        self.content = Some(content.into_view_fn());
        self
    }
    /// Make details collapsible.
    pub fn collapsible(mut self, value: impl IntoValue<bool>) -> Self {
        self.collapsible = value.into_value();
        self
    }
    /// Bind expanded details to application state.
    pub fn open(mut self, value: impl IntoBinding<bool>) -> Self {
        self.open = Some(value.into_binding());
        self
    }
    /// Show a dismiss control; dismissal lasts until remount.
    pub fn dismissible(mut self, value: impl IntoValue<bool>) -> Self {
        self.dismissible = value.into_value();
        self
    }
    /// Name the dismiss control.
    pub fn dismiss_label(mut self, label: impl IntoText) -> Self {
        self.dismiss_label = label.into_text_value();
        self
    }
    /// Replace the visible dismiss contents, preserving the accessible dismiss label.
    pub fn dismiss_content(mut self, content: impl IntoView) -> Self {
        self.dismiss_content = Some(content.into_view());
        self
    }
    /// Show a dismiss control and report dismissal after restoring focus.
    pub fn on_dismiss(mut self, f: impl Into<Callback<()>>) -> Self {
        self.dismissible = true.into_value();
        self.dismiss = Some(f.into());
        self
    }
    /// Use application-owned visibility. A refused false write keeps the alert visible;
    /// completed dismissal and focus restoration happen only after acceptance.
    pub fn visible(mut self, visible: impl IntoBinding<bool>) -> Self {
        self.visible = Some(visible.into_binding());
        self
    }
    /// Set a structural or theme-defined appearance.
    pub fn variant(mut self, value: impl Variant) -> Self {
        self.variants.add(value);
        self
    }
}
impl NodeBuilder for Alert {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Alert {
    fn into_view(mut self) -> View {
        let reference = node_ref();
        let phase = banner_dismissal(reference, self.dismiss.take(), self.visible);
        let tone = self.tone;
        let dismissible = self.dismissible;
        let collapsible = self.collapsible;
        let has_details = self.content.is_some();
        let open = self.open.unwrap_or_else(|| signal(false).into_binding());
        let has_icon = self.icon.is_some();
        let has_actions = self.actions.is_some();
        let has_description = self.description.is_some();
        let state = move || AlertState {
            tone: tone.get(),
            expanded: (has_details && collapsible.get()).then(|| open.get()),
            dismissible: dismissible.get(),
            has_icon,
            has_actions,
            has_description,
            has_content: has_details,
        };
        let contents = self.render_contents(state, collapsible, open, phase);
        with_base_recipe_style(
            self.root,
            |r| r.alert_recipes().root_recipe(),
            &self.variants,
            state,
        )
        .node_ref(reference)
        .hidden(move || !phase.state().get().visible())
        .child(contents)
        .into_view()
    }
}

impl Alert {
    fn render_contents(
        &mut self,
        state: impl Fn() -> AlertState + Copy + 'static,
        collapsible: Value<bool>,
        open: Binding<bool>,
        phase: MachineHandle<BannerMachine>,
    ) -> View {
        let variants = &self.variants;
        let icon = self.icon.take().map(|view| {
            with_recipe_style(
                node().child(view),
                |r| r.alert_recipes().icon_recipe(),
                variants,
                state,
            )
        });
        let title = with_recipe_style(
            node().child(text(self.title)),
            |r| r.alert_recipes().title_recipe(),
            variants,
            state,
        );
        let description = self.description.take().map(|value| {
            with_recipe_style(
                text(value),
                |r| r.alert_recipes().description_recipe(),
                variants,
                state,
            )
        });
        let actions = self.actions.take().map(|view| {
            with_recipe_style(
                node().child(view),
                |r| r.alert_recipes().actions_recipe(),
                variants,
                state,
            )
        });
        let toggle = expansion_toggle(self.content.is_some(), open, collapsible, variants, state);
        let content = self.content.take().map(|content| {
            let variants = self.variants.clone();
            show(
                move || !collapsible.get() || open.get(),
                move || {
                    with_recipe_style(
                        node().child(content.call(())),
                        |r| r.alert_recipes().content_recipe(),
                        &variants,
                        state,
                    )
                },
            )
        });
        let dismissible = self.dismissible;
        let dismiss_content = self.dismiss_content.take();
        let dismiss = with_recipe_style(
            super::button(self.dismiss_label)
                .hide_label(dismiss_content.is_some())
                .child(dismiss_content)
                .on_press(move || phase.send(BannerEvent::Dismiss))
                .hidden(move || !dismissible.get()),
            |r| r.alert_recipes().dismiss_recipe(),
            variants,
            state,
        );
        (
            alert_header(
                icon,
                (title, description),
                (actions, toggle, dismiss),
                variants,
                state,
            ),
            content,
        )
            .into_view()
    }
}
/// Present actual slots without adding behavior or changing their semantic targets.
fn alert_header(
    icon: impl IntoView,
    message: impl IntoView,
    controls: impl IntoView,
    variants: &Variants,
    state: impl Fn() -> AlertState + Copy + 'static,
) -> Node {
    let message = with_recipe_style(
        node().child(message),
        |r| r.alert_recipes().message_recipe(),
        variants,
        state,
    );
    let controls = with_recipe_style(
        node().child(controls),
        |r| r.alert_recipes().controls_recipe(),
        variants,
        state,
    );
    with_recipe_style(
        node().child((icon, message, controls)),
        |r| r.alert_recipes().header_recipe(),
        variants,
        state,
    )
}
fn expansion_toggle(
    has_details: bool,
    open: Binding<bool>,
    collapsible: Value<bool>,
    variants: &Variants,
    state: impl Fn() -> AlertState + Copy + 'static,
) -> Option<impl IntoView> {
    has_details.then(|| {
        let expand = kivact_core::i18n::message("alert.expand", "Expand details");
        let collapse = kivact_core::i18n::message("alert.collapse", "Collapse details");
        let label = kivact_core::state::memo(move || {
            if open.get() {
                collapse.get()
            } else {
                expand.get()
            }
        });
        let control = super::button(label)
            .with_node(move |node| node.expanded(move || open.get()))
            .hidden(move || !collapsible.get())
            .on_press(move || open.set(!open.get()));
        crate::tooltip(label).attach(with_recipe_style(
            control,
            |r| r.alert_recipes().toggle_recipe(),
            variants,
            state,
        ))
    })
}

enum BannerEvent {
    ExternalFocus {
        focused: Option<NodeToken>,
        within: bool,
    },
    Dismiss,
}
enum BannerEffect {
    RestoreFocus(NodeToken),
    NotifyDismiss,
    RequestDismiss {
        visible: Binding<bool>,
        restore: Option<NodeToken>,
    },
}
struct BannerMachine;
impl Machine for BannerMachine {
    type State = BannerState;
    type Event = BannerEvent;
    type Effect = BannerEffect;
    fn transition(
        &self,
        state: &BannerState,
        event: BannerEvent,
    ) -> Step<BannerState, BannerEvent, BannerEffect> {
        match (*state, event) {
            (
                state,
                BannerEvent::ExternalFocus {
                    focused: Some(target),
                    within: false,
                },
            ) if state.visible() => Step::to(match state {
                BannerState::Local(_) => BannerState::Local(LocalBannerState::Shown {
                    restore: Some(target),
                }),
                BannerState::Shared { visible, .. } => BannerState::Shared {
                    visible,
                    restore: Some(target),
                },
            }),
            (BannerState::Local(LocalBannerState::Shown { restore }), BannerEvent::Dismiss) => {
                let step = Step::to(BannerState::Local(LocalBannerState::Dismissed));
                let step = if let Some(target) = restore {
                    step.with(BannerEffect::RestoreFocus(target))
                } else {
                    step
                };
                step.with(BannerEffect::NotifyDismiss)
            }
            (BannerState::Shared { visible, restore }, BannerEvent::Dismiss) if visible.get() => {
                Step::stay().with(BannerEffect::RequestDismiss { visible, restore })
            }
            _ => Step::stay(),
        }
    }
}
fn banner_dismissal(
    reference: NodeRef,
    callback: Option<Callback<()>>,
    visible: Option<Binding<bool>>,
) -> MachineHandle<BannerMachine> {
    let phase = machine(
        visible.map_or(
            BannerState::Local(LocalBannerState::Shown { restore: None }),
            |visible| BannerState::Shared {
                visible,
                restore: None,
            },
        ),
        || BannerMachine,
        move |_, command| match command {
            BannerEffect::RestoreFocus(target) => target.restore_focus(),
            BannerEffect::RequestDismiss { visible, restore } => {
                visible.set(false);
                if !visible.get_untracked() {
                    if let Some(target) = restore {
                        target.restore_focus();
                    }
                    if let Some(callback) = callback {
                        callback.call(());
                    }
                }
            }
            BannerEffect::NotifyDismiss => {
                if let Some(callback) = callback {
                    callback.call(());
                }
            }
        },
    );
    let owner = Owner::current();
    let within = reference.focus_within();
    effect(move || {
        if let Some(visible) = visible {
            visible.get();
        }
        phase.send(BannerEvent::ExternalFocus {
            focused: owner.focused_node(),
            within: within.get(),
        });
    });
    phase
}
