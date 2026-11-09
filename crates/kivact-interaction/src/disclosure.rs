//! Role: DisclosureApi attaches an expandable trigger and its optionally delayed content.
//!
//! A disclosure is the first link of the opening chain: a popup
//! (`crate::popup`) adds overlay rules to it, and a tooltip and a dialog add
//! theirs to a popup. Its shown state is the requested state, the caller's
//! `Binding<bool>`; [`DisclosureApi::close`] writes `false` to it. A delayed
//! shown state is a [`DelayedApi`](crate::DelayedApi).
//! The parts are the trigger and the content, with [`DisclosureState`]
//! (`input`, `open`). Mount content with `show` so engine presence owns
//! enter and exit paint; the trigger follows the requested state immediately.

use crate::options_cell::OptionsCell;
use kivact_core::custom::{Part, PartState, part};
use kivact_core::state::IntoValue;
use kivact_core::state::{Binding, Signal, Value, on_cleanup, signal};
use kivact_core::view::{A11yAction, NodeRef, Role, node_ref};

/// The shown state and actual input state of a disclosure's trigger or content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisclosureState {
    pub input: PartState,
    /// The content is shown (after the delay, when there is one).
    pub open: bool,
}

/// Copy handle for a collapsible region: a trigger and the content it shows.
///
/// The disclosure edits one `bool`, the requested open state, through the
/// caller's [`Binding`]. The caller draws the trigger and the content and
/// decides whether the content is unmounted (`show(d.is_open(), ..)`) or kept
/// mounted; [`content`](Self::content) hides mounted content while closed.
///
/// The [`trigger`](Self::trigger) is a focusable `Role::Button` publishing
/// `expanded` and `controls` to the content. A click, Enter, Space and the
/// accessibility actions `Activate` and `Toggle` flip the state, while `Expand`
/// sets it and `Collapse` clears it. A [`disabled`](Self::disabled)
/// disclosure accepts no input or assistive technology action.
///
/// ```no_run
/// use kivact_core::{state::signal, view::{Element, column, node, text, show}};
/// use kivact_interaction::use_disclosure;
/// let open = signal(false);
/// let details = use_disclosure(open);
/// let section = column((
///     node().part(details.trigger()).name("Details").child(text("Details")),
///     show(details.is_open(), move || node().part(details.content()).child(text("The details"))),
/// ));
/// ```
#[derive(Clone, Copy)]
pub struct DisclosureApi {
    pub(crate) open: Binding<bool>,
    /// The shown state: the requested state.
    shown: Value<bool>,
    // xtask-allow(state-fields): independent fact - the observation of the attached trigger part, absent until it attaches
    trigger_state: Signal<Option<Value<DisclosureState>>>,
    // xtask-allow(state-fields): independent fact - the observation of the attached content part, absent until it attaches
    content_state: Signal<Option<Value<DisclosureState>>>,
    settings: OptionsCell<DisclosureOptions>,
    /// The disabled state each mounted trigger received from its caller;
    /// each entry ends with its trigger's mount.
    host_disabled: Signal<Vec<Value<bool>>>,
    pub(crate) content: NodeRef,
}

/// What the setters store; read where the parts use it, so they work in any
/// order.
#[derive(Clone, Copy, Default)]
struct DisclosureOptions {
    disabled: Option<Value<bool>>,
}

/// Create a disclosure whose requested state is `open`, the sole writer.
///
/// Creation does not write the state. Setup only.
#[track_caller]
pub fn use_disclosure(open: impl kivact_core::state::IntoBinding<bool>) -> DisclosureApi {
    let open = open.into_binding();
    DisclosureApi {
        open,
        shown: open.into_value(),
        trigger_state: signal(None),
        content_state: signal(None),
        settings: OptionsCell::new(DisclosureOptions::default()),
        host_disabled: signal(Vec::new()),
        content: node_ref(),
    }
}

impl DisclosureApi {
    /// Drawing state of the mounted trigger.
    pub fn trigger_state(self) -> Value<DisclosureState> {
        kivact_core::state::memo(move || {
            self.trigger_state
                .get()
                .map(|state| state.get())
                .unwrap_or(DisclosureState {
                    input: PartState::default(),
                    open: self.shown.get(),
                })
        })
    }
    /// Drawing state of the mounted content.
    pub fn content_state(self) -> Value<DisclosureState> {
        kivact_core::state::memo(move || {
            self.content_state
                .get()
                .map(|state| state.get())
                .unwrap_or(DisclosureState {
                    input: PartState::default(),
                    open: self.shown.get(),
                })
        })
    }

    /// Ignore input and assistive technology while true.
    ///
    /// Also marks the trigger disabled and makes it unfocusable. Setup only.
    #[track_caller]
    pub fn disabled(self, disabled: impl IntoValue<bool>) -> Self {
        let disabled = disabled.into_value();
        self.settings
            .update(|settings| settings.disabled = Some(disabled));
        self
    }

    /// Whether the content is shown: the requested state.
    pub fn is_open(self) -> Value<bool> {
        self.shown
    }

    /// Close: write `false` to the binding.
    #[track_caller]
    pub fn close(self) {
        self.open.set(false);
    }

    fn is_disabled(self) -> bool {
        self.settings.get().disabled.is_some_and(|d| d.get())
            || self
                .host_disabled
                .with_untracked(|hosts| hosts.iter().any(|d| d.get_untracked()))
    }

    /// Register the disabled state a mounted trigger received from its
    /// caller until that mount's owner ends.
    fn register_host_disabled(self, disabled: Option<Value<bool>>) {
        let Some(disabled) = disabled else {
            return;
        };
        let hosts = self.host_disabled;
        hosts.update(|hosts| hosts.push(disabled));
        on_cleanup(move || {
            if hosts.try_get_untracked().is_some() {
                hosts.update(|hosts| {
                    if let Some(at) = hosts.iter().position(|host| *host == disabled) {
                        hosts.remove(at);
                    }
                });
            }
        });
    }

    fn write(self, open: Option<bool>) {
        if self.is_disabled() {
            return;
        }
        self.open.set(open.unwrap_or_else(|| !self.open.get()));
    }

    /// Trigger semantics shared with the popups built on a disclosure.
    #[track_caller]
    pub(crate) fn base_trigger<S: kivact_core::state::PropValue>(
        self,
        state: impl Fn(PartState, bool) -> S + 'static,
    ) -> Part<S> {
        let shown = self.shown;
        let settings = self.settings;
        let hosts = self.host_disabled;
        let disabled = move || {
            settings
                .value()
                .get()
                .disabled
                .is_some_and(|disabled| disabled.get())
                || hosts.with(|hosts| hosts.iter().any(|disabled| disabled.get()))
        };
        let input = part(move |input| state(input, shown.get()));
        let part_state = input.state();
        Part::new(part_state, move |mut host| {
            self.register_host_disabled(host.accessible_disabled());
            host.part(input)
                .role(Role::Button)
                .expanded(shown)
                .controls(self.content)
                .disabled(disabled)
                .focusable(move || !disabled())
        })
    }

    /// The trigger: a button publishing `expanded` that toggles the state.
    ///
    /// The trigger's name, geometry and drawing stay the caller's. Activate,
    /// Toggle, Expand and Collapse run after the caller's own handlers.
    #[track_caller]
    pub fn trigger(self) -> Part<DisclosureState> {
        let base = self.base_trigger(|input, open| DisclosureState { input, open });
        let part_state = base.state();
        self.trigger_state.set(Some(part_state));
        Part::new(part_state, move |host| {
            host.part(base)
                .on_press(move || self.write(None))
                .a11y_action(A11yAction::Toggle, move |_| self.write(None))
                .a11y_action(A11yAction::Expand, move |_| self.write(Some(true)))
                .a11y_action(A11yAction::Collapse, move |_| self.write(Some(false)))
        })
    }

    /// The content the trigger `controls`, hidden while closed. With
    /// `show(is_open(), ..)`, engine presence retains exit paint after unmount.
    #[track_caller]
    pub fn content(self) -> Part<DisclosureState> {
        let shown = self.shown;
        let input = part(move |input| DisclosureState {
            input,
            open: shown.get(),
        });
        let part_state = input.state();
        self.content_state.set(Some(part_state));
        let target = kivact_core::view::node_ref();
        let presence = target.presence();
        Part::new(part_state, move |host| {
            host.part(input)
                .node_ref(target)
                .node_ref(self.content)
                .hidden(move || {
                    !shown.get() && presence.get() != kivact_core::view::Presence::Leaving
                })
        })
    }
}
