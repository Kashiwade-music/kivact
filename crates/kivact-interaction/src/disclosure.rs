//! Role: DisclosureApi attaches an expandable trigger and its optionally delayed content.
//!
//! A disclosure is the first link of the opening chain: a popup
//! (`crate::popup`) adds overlay rules to it, and a tooltip and a dialog add
//! theirs to a popup. Its shown state is the requested state, the caller's
//! `Binding<bool>`; [`DisclosureApi::close`] writes `false` to it. A delayed
//! shown state is a [`DelayedApi`](crate::DelayedApi).
//! The parts are the trigger and the content, with [`DisclosureState`]
//! (`input`, `open`). Whether the content is unmounted with `show` or kept
//! mounted and hidden is the caller's choice; the kits' toggles and
//! accordions share these rules. A content that animates its closing (an
//! accordion's height) stays visible while [`DisclosureApi::leaving`] says the
//! animation runs; the trigger's `expanded` follows the shown state at once.

use kivact_core::custom::{Part, PartState, part};
use kivact_core::state::IntoBinding;
use kivact_core::state::IntoValue;
use kivact_core::state::{Binding, Signal, Value, signal};
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
/// mounted; [`content`](Self::content) hides a mounted content while closed.
///
/// The [`trigger`](Self::trigger) is a focusable `Role::Button` publishing
/// `expanded` and `controls` to the content. A click, Enter, Space and the
/// accessibility actions `Activate` and `Toggle` flip the state, while `Expand`
/// sets it and `Collapse` clears it. A [`disabled`](Self::disabled)
/// disclosure accepts no input or assistive technology action.
///
/// ```no_run
/// use kivact_core::{state::signal, view::{NodeBuilder, column, node, text, show}};
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
    trigger_state: Signal<Option<Value<DisclosureState>>>,
    content_state: Signal<Option<Value<DisclosureState>>>,
    settings: Signal<DisclosureSettings>,
    pub(crate) content: NodeRef,
}

/// What the setters store; read where the parts use it, so they work in any
/// order.
#[derive(Clone, Copy, Default)]
struct DisclosureSettings {
    disabled: Option<Value<bool>>,
    leaving: Option<Value<bool>>,
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
        settings: signal(DisclosureSettings::default()),
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

    /// Keep the [`content`](Self::content) visible after it stops being shown
    /// while `leaving` is true: the caller's closing animation (a height that
    /// collapses) still runs. The trigger's `expanded` and the content's
    /// [`DisclosureState::open`] follow the shown state at once. Setup only.
    pub fn leaving(self, leaving: impl IntoValue<bool>) -> Self {
        let leaving = leaving.into_value();
        self.settings
            .update(|settings| settings.leaving = Some(leaving));
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
        self.settings
            .with_untracked(|settings| settings.disabled.is_some_and(|d| d.get()))
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
        let disabled = move || {
            settings.with(|settings| settings.disabled.is_some_and(|disabled| disabled.get()))
        };
        let input = part(move |input| state(input, shown.get()));
        let part_state = input.state();
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |mut host| {
            let caller_disabled = host.accessible_disabled();
            let own_disabled = settings.get_untracked().disabled;
            settings.update(|settings| {
                settings.disabled = Some(kivact_core::state::memo(move || {
                    caller_disabled.is_some_and(|value| value.get())
                        || own_disabled.is_some_and(|value| value.get())
                }))
            });
            host.part(input)
                .node_ref(target)
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

    /// The content the trigger `controls`, hidden while not shown (nor
    /// [`leaving`](Self::leaving)).
    #[track_caller]
    pub fn content(self) -> Part<DisclosureState> {
        let shown = self.shown;
        let settings = self.settings;
        let input = part(move |input| DisclosureState {
            input,
            open: shown.get(),
        });
        let part_state = input.state();
        self.content_state.set(Some(part_state));
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |host| {
            host.part(input)
                .node_ref(target)
                .node_ref(self.content)
                .hidden(move || {
                    let leaving = settings
                        .with(|settings| settings.leaving)
                        .is_some_and(|leaving| leaving.get());
                    !shown.get() && !leaving
                })
        })
    }
}
