//! Role: The Alert builder, its rendering, and the dismissal state machine.

use crate::alert::look::AlertState;
use crate::alert::look::AlertTheme;
use crate::alert::look::Expansion;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use kivact_core::custom::{NodeToken, Owner};
use kivact_core::state::IntoHandler;
use kivact_core::state::{
    Callback, IntoText, IntoValue, Machine, MachineHandle, Signal, Step, Value, effect, machine,
    signal,
};
use kivact_core::view::Variants;
use kivact_core::view::{
    Element, IntoView, IntoViewFn, LiveRegion, Node, NodeRef, Role, View, ViewFn, node, node_ref,
    show, text,
};
use kivact_interaction::Tone;

#[derive(Clone, Copy, PartialEq)]
enum BannerState {
    /// Visible; `restore` is the last focused node outside the banner while it was visible.
    Shown { restore: Option<NodeToken> },
    /// Dismissed by the user (local) or hidden by the application's binding.
    Hidden,
}
impl BannerState {
    fn visible(self) -> bool {
        matches!(self, Self::Shown { .. })
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Ownership {
    /// The alert owns its visibility; dismissal lasts until remount.
    Local,
    /// The application's `open` binding owns the visibility.
    Application,
}

// xtask-allow(state): legacy: description, icon, actions, collapsible, dismiss_content, open
/// An urgent, persistent message with optional actions and details.
pub struct Alert {
    root: Node,
    title: Value<String>,
    icon: Option<View>,
    actions: Option<View>,
    content: Option<ViewFn>,
    collapsible: bool,
    tone: Value<Tone>,
    dismissible: Value<bool>,
    dismiss_label: Value<String>,
    dismiss_content: Option<View>,
    dismiss: Option<Callback<()>>,
    open: Option<Signal<bool>>,
}
/// Display an alert without moving keyboard focus or automatically hiding it.
#[track_caller]
pub fn alert(title: impl IntoText) -> Alert {
    let title = title.into_text_value();
    Alert {
        root: node()
            .role(Role::Alert)
            .live_region(LiveRegion::Assertive)
            .name(title),
        title,
        icon: None,
        actions: None,
        content: None,
        collapsible: false,
        tone: Tone::Neutral.into_value(),
        dismissible: false.into_value(),
        dismiss_label: "Dismiss notification".into_text_value(),
        dismiss_content: None,
        dismiss: None,
        open: None,
    }
}
impl Alert {
    /// Set the semantic tone.
    #[track_caller]
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
    /// Make details collapsible. It is fixed for the mounted alert, since it
    /// decides what [`open`](Self::open) controls.
    pub fn collapsible(mut self, value: bool) -> Self {
        self.collapsible = value;
        self
    }
    /// Show a dismiss control; dismissal lasts until remount.
    #[track_caller]
    pub fn dismissible(mut self, value: impl IntoValue<bool>) -> Self {
        self.dismissible = value.into_value();
        self
    }
    /// Name the dismiss control.
    #[track_caller]
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
    #[track_caller]
    pub fn on_dismiss<M>(mut self, f: impl IntoHandler<(), M>) -> Self {
        self.dismissible = true.into_value();
        self.dismiss = Some(kivact_core::state::Callback::from_handler(
            kivact_core::state::IntoHandler::into_handler(f),
        ));
        self
    }
    /// Use application-owned open state: the expanded details when the alert
    /// has collapsible details, otherwise the alert's visibility. A refused
    /// false visibility write keeps the alert visible; completed dismissal and
    /// focus restoration happen only after acceptance.
    pub fn open(mut self, open: Signal<bool>) -> Self {
        self.open = Some(open);
        self
    }
}
impl Element for Alert {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Alert {
    fn into_view(mut self) -> View {
        let variants = self.root.authored().variants();
        let reference = node_ref();
        let tone = self.tone;
        let dismissible = self.dismissible;
        let fixed = self.collapsible;
        let collapsible: Value<bool> = fixed.into_value();
        let has_details = self.content.is_some();
        // The alert has one open state: its disclosure when it has collapsible
        // details, otherwise its visibility.
        let (details, visible) = if has_details && fixed {
            (self.open.unwrap_or_else(|| signal(false)), None)
        } else {
            (signal(false), self.open)
        };
        let phase = banner_dismissal(reference, self.dismiss.take(), visible);
        let has_icon = self.icon.is_some();
        let has_actions = self.actions.is_some();
        let has_description = self.root.authored().description().is_some();
        let state = move || AlertState {
            tone: tone.get(),
            content: has_details.then(|| Expansion::new(collapsible.get(), details.get())),
            dismissible: dismissible.get(),
            has_icon,
            has_actions,
            has_description,
        };
        let contents = self.render_contents(state, collapsible, details, phase);
        with_base_recipe_style(
            self.root,
            |r: &AlertTheme| r.root_recipe(),
            &variants,
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
        open: Signal<bool>,
        phase: MachineHandle<BannerMachine>,
    ) -> View {
        let variants = self.root.authored().variants();
        let variants = &variants;
        let icon = self.icon.take().map(|view| {
            with_recipe_style(
                node().child(view),
                |r: &AlertTheme| r.icon_recipe(),
                variants,
                state,
            )
        });
        let title = with_recipe_style(
            node().child(text(self.title)),
            |r: &AlertTheme| r.title_recipe(),
            variants,
            state,
        );
        let description = self.root.authored().description().map(|value| {
            with_recipe_style(
                text(value),
                |r: &AlertTheme| r.description_recipe(),
                variants,
                state,
            )
        });
        let actions = self.actions.take().map(|view| {
            with_recipe_style(
                node().child(view),
                |r: &AlertTheme| r.actions_recipe(),
                variants,
                state,
            )
        });
        let toggle = expansion_toggle(self.content.is_some(), open, collapsible, variants, state);
        let content = self.content.take().map(|content| {
            let variants = variants.clone();
            show(
                move || !collapsible.get() || open.get(),
                move || {
                    with_recipe_style(
                        node().child(content.call(())),
                        |r: &AlertTheme| r.content_recipe(),
                        &variants,
                        state,
                    )
                },
            )
        });
        let dismissible = self.dismissible;
        let dismiss_content = self.dismiss_content.take();
        let dismiss = with_recipe_style(
            crate::button(self.dismiss_label)
                .hide_label(dismiss_content.is_some())
                .children((dismiss_content,))
                .on_press(move || phase.send(BannerEvent::Dismiss))
                .hidden(move || !dismissible.get()),
            |r: &AlertTheme| r.dismiss_recipe(),
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
        |r: &AlertTheme| r.message_recipe(),
        variants,
        state,
    );
    let controls = with_recipe_style(
        node().child(controls),
        |r: &AlertTheme| r.controls_recipe(),
        variants,
        state,
    );
    with_recipe_style(
        node().child((icon, message, controls)),
        |r: &AlertTheme| r.header_recipe(),
        variants,
        state,
    )
}
fn expansion_toggle(
    has_details: bool,
    open: Signal<bool>,
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
        let control = crate::button(label)
            .with_node(move |node| node.expanded(move || open.get()))
            .hidden(move || !collapsible.get())
            .on_press(move || open.set(!open.get()));
        crate::tooltip(
            label,
            with_recipe_style(control, |r: &AlertTheme| r.toggle_recipe(), variants, state),
        )
    })
}

enum BannerEvent {
    /// One snapshot: the application's visibility (None when the alert owns it)
    /// and the focused node with whether it lies inside the banner.
    Observed {
        shown: Option<bool>,
        focused: Option<NodeToken>,
        within: bool,
    },
    Dismiss,
}
enum BannerEffect {
    RestoreFocus(NodeToken),
    NotifyDismiss,
    RequestDismiss { restore: Option<NodeToken> },
}
struct BannerMachine {
    ownership: Ownership,
}
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
                BannerEvent::Observed {
                    shown,
                    focused,
                    within,
                },
            ) => {
                let outside = focused.filter(|_| !within);
                match (state, shown) {
                    (BannerState::Shown { .. }, Some(false)) => Step::to(BannerState::Hidden),
                    (BannerState::Hidden, Some(true)) => {
                        Step::to(BannerState::Shown { restore: outside })
                    }
                    (BannerState::Shown { .. }, _) if outside.is_some() => {
                        Step::to(BannerState::Shown { restore: outside })
                    }
                    _ => Step::stay(),
                }
            }
            (BannerState::Shown { restore }, BannerEvent::Dismiss) => match self.ownership {
                Ownership::Local => {
                    let step = Step::to(BannerState::Hidden);
                    let step = match restore {
                        Some(target) => step.with(BannerEffect::RestoreFocus(target)),
                        None => step,
                    };
                    step.with(BannerEffect::NotifyDismiss)
                }
                Ownership::Application => {
                    Step::stay().with(BannerEffect::RequestDismiss { restore })
                }
            },
            (BannerState::Hidden, BannerEvent::Dismiss) => Step::stay(),
        }
    }
}
fn banner_dismissal(
    reference: NodeRef,
    callback: Option<Callback<()>>,
    visible: Option<Signal<bool>>,
) -> MachineHandle<BannerMachine> {
    let initial = match visible {
        Some(visible) if !visible.get_untracked() => BannerState::Hidden,
        _ => BannerState::Shown { restore: None },
    };
    let ownership = if visible.is_some() {
        Ownership::Application
    } else {
        Ownership::Local
    };
    let phase = machine(
        initial,
        move || BannerMachine { ownership },
        move |_, command| match command {
            BannerEffect::RestoreFocus(target) => target.restore_focus(),
            BannerEffect::RequestDismiss { restore } => {
                let Some(visible) = visible else { return };
                // The binding may already be false before this snapshot arrived.
                if !visible.get_untracked() {
                    return;
                }
                visible.set(false);
                // A refused write keeps the alert visible: no focus move, no callback.
                if visible.get_untracked() {
                    return;
                }
                if let Some(target) = restore {
                    target.restore_focus();
                }
                if let Some(callback) = callback {
                    callback.call(());
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
        phase.send(BannerEvent::Observed {
            shown: visible.map(|visible| visible.get()),
            focused: owner.focused_node(),
            within: within.get(),
        });
    });
    phase
}
