//! Role: Present persistent alerts and dismissible feedback banners.

pub(crate) mod look;

pub use look::AlertVariant;

use crate::alert::look::{AlertState, AlertTheme, Expansion};
use crate::layer::{with_base_recipe_style, with_recipe_style};
use kivact_core::custom::{Caller, NodeToken, Owner};
use kivact_core::state::{
    Callback, IntoValue, Machine, MachineHandle, Signal, Step, Value, effect, machine, signal,
};
use kivact_core::view::{
    Element, IntoView, LiveRegion, Node, NodeRef, Role, Variants, View, ViewFn, node, node_ref,
    show, text,
};
use kivact_interaction::Tone;
use kivact_macros::component;

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

/// An urgent, persistent message with optional actions and details.
///
/// `dismissible` shows a dismiss control; `on_dismiss` also shows it. The
/// details of `content` collapse only with `collapsible`, and `open` drives
/// the details then, or the visibility of the alert otherwise.
#[component]
pub fn alert(
    title: Value<String>,
    #[prop(default = Tone::Neutral)] tone: Value<Tone>,
    #[prop(default = false)] dismissible: Value<bool>,
    #[prop(default = "Dismiss notification")] dismiss_label: Value<String>,
    icon: Option<View>,
    actions: Option<View>,
    content: Option<ViewFn>,
    #[prop(default = false)] collapsible: bool,
    dismiss_content: Option<View>,
    on_dismiss: Option<Callback<()>>,
    open: Option<Signal<bool>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let reference = node_ref();
    let fixed = collapsible;
    let collapsible: Value<bool> = fixed.into_value();
    let has_details = content.is_some();
    // The alert has one open state: its disclosure when it has collapsible
    // details, otherwise its visibility.
    let (details, visible) = if has_details && fixed {
        (open.unwrap_or_else(|| signal(false)), None)
    } else {
        (signal(false), open)
    };
    // A dismiss handler makes the alert dismissible.
    let dismissible: Value<bool> = if on_dismiss.is_some() {
        true.into_value()
    } else {
        dismissible
    };
    let phase = banner_dismissal(reference, on_dismiss, visible);
    let (has_icon, has_actions) = (icon.is_some(), actions.is_some());
    let has_description = caller.description().is_some();
    let state = move || AlertState {
        tone: tone.get(),
        content: has_details.then(|| Expansion::new(collapsible.get(), details.get())),
        dismissible: dismissible.get(),
        has_icon,
        has_actions,
        has_description,
    };
    let parts = AlertParts {
        icon,
        title,
        description: caller.description(),
        actions,
        content,
        dismissible,
        dismiss_label,
        dismiss_content,
    };
    let contents = render_contents(parts, &variants, state, collapsible, details, phase);
    with_base_recipe_style(
        node()
            .role(Role::Alert)
            .live_region(LiveRegion::Assertive)
            .name(title),
        |r: &AlertTheme| r.root_recipe(),
        &variants,
        state,
    )
    .node_ref(reference)
    .hidden(move || !phase.state().get().visible())
    .child(contents)
}

/// The slots and the dismissal of an alert, as its body gives them over.
struct AlertParts {
    icon: Option<View>,
    title: Value<String>,
    description: Option<Value<String>>,
    actions: Option<View>,
    content: Option<ViewFn>,
    dismissible: Value<bool>,
    dismiss_label: Value<String>,
    dismiss_content: Option<View>,
}

fn render_contents(
    parts: AlertParts,
    variants: &Variants,
    state: impl Fn() -> AlertState + Copy + 'static,
    collapsible: Value<bool>,
    open: Signal<bool>,
    phase: MachineHandle<BannerMachine>,
) -> View {
    let icon = parts.icon.map(|view| {
        with_recipe_style(
            node().child(view),
            |r: &AlertTheme| r.icon_recipe(),
            variants,
            state,
        )
    });
    let title = with_recipe_style(
        node().child(text(parts.title)),
        |r: &AlertTheme| r.title_recipe(),
        variants,
        state,
    );
    let description = parts.description.map(|value| {
        with_recipe_style(
            text(value),
            |r: &AlertTheme| r.description_recipe(),
            variants,
            state,
        )
    });
    let actions = parts.actions.map(|view| {
        with_recipe_style(
            node().child(view),
            |r: &AlertTheme| r.actions_recipe(),
            variants,
            state,
        )
    });
    let toggle = expansion_toggle(parts.content.is_some(), open, collapsible, variants, state);
    let content = parts.content.map(|content| {
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
    let dismissible = parts.dismissible;
    let dismiss_content = parts.dismiss_content;
    let dismiss = with_recipe_style(
        crate::button(parts.dismiss_label)
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
