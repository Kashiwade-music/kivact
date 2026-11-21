//! Role: DisclosureApi attaches an expandable trigger and its optionally delayed content.
//!
//! A disclosure is the first link of the opening chain: a popup
//! (`crate::popup`) adds overlay rules to it, and a tooltip and a dialog add
//! theirs to a popup. Its shown state is the requested state, the caller's
//! `Signal<bool>`; [`show`](DisclosureApi::show), [`hide`](DisclosureApi::hide)
//! and [`toggle`](DisclosureApi::toggle) write it. A delayed
//! shown state is a [`DelayedApi`](crate::DelayedApi).
//! The parts are the trigger and the content, with [`DisclosureState`]
//! (`input`, `open`). Mount content with `show` so engine presence owns
//! enter and exit paint; the trigger follows the requested state immediately.

use kivact_core::custom::{Part, PartState, part};
use kivact_core::state::IntoValue;
use kivact_core::state::{Signal, Value};
use kivact_core::view::{A11yAction, Element, IntoView, NodeRef, Placement, Role, View, node_ref};

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
/// caller's [`Signal`]. The caller draws the trigger and the content and
/// decides whether the content is unmounted (`show(d.is_open(), ..)`) or kept
/// mounted; [`content`](Self::content) hides mounted content while closed.
///
/// The [`trigger`](Self::trigger) is a focusable `Role::Button` publishing
/// `expanded` and `controls` to the content. A click, Enter, Space and the
/// accessibility actions `Activate` and `Toggle` flip the state, while `Expand`
/// sets it and `Collapse` clears it. A trigger node made `.disabled(..)`
/// before the part is attached accepts no input or assistive technology action.
///
/// ```no_run
/// use kivact::{state::signal, view::{Element, column, node, text, show}};
/// use kivact::interaction::use_disclosure;
/// let open = signal(false);
/// let details = use_disclosure(open);
/// let section = column((
///     node().part(details.trigger()).name("Details").child(text("Details")),
///     show(details.is_open(), move || node().part(details.content()).child(text("The details"))),
/// ));
/// ```
#[derive(Clone, Copy)]
pub struct DisclosureApi {
    pub(crate) open: Signal<bool>,
    /// The shown state: the requested state.
    shown: Value<bool>,
    /// The node the trigger part binds; a surface is placed at it.
    anchor: NodeRef,
    pub(crate) content: NodeRef,
}

/// Create a disclosure whose requested state is `open`, the sole writer.
///
/// Creation does not write the state. Setup only.
#[track_caller]
pub fn use_disclosure(open: kivact_core::state::Signal<bool>) -> DisclosureApi {
    DisclosureApi {
        open,
        shown: open.into_value(),
        anchor: node_ref(),
        content: node_ref(),
    }
}

/// The disabled state the caller gave the trigger node with `.disabled(..)`
/// before attaching the part: the one source of a trigger's disabled state.
fn trigger_disabled(host: &mut kivact_core::view::Node) -> Value<bool> {
    host.accessible_disabled()
        .unwrap_or_else(|| false.into_value())
}

impl DisclosureApi {
    /// Whether the content is shown: the requested state.
    pub fn is_open(self) -> Value<bool> {
        self.shown
    }

    /// Show: write `true` unless it is shown.
    pub fn show(self) {
        if !self.open.get_untracked() {
            self.open.set(true);
        }
    }

    /// Hide: write `false` unless it is hidden.
    pub fn hide(self) {
        if self.open.get_untracked() {
            self.open.set(false);
        }
    }

    /// Invert the requested state.
    pub fn toggle(self) {
        self.open.set(!self.open.get_untracked());
    }

    fn write(self, open: Option<bool>) {
        self.open
            .set(open.unwrap_or_else(|| !self.open.get_untracked()));
    }

    /// Trigger semantics shared with the popups built on a disclosure.
    #[track_caller]
    pub(crate) fn base_trigger<S: kivact_core::state::PropValue>(
        self,
        state: impl Fn(PartState, bool) -> S + 'static,
    ) -> Part<S> {
        let shown = self.shown;
        let input = part(move |input| state(input, shown.get()));
        let part_state = input.state();
        Part::new(part_state, move |mut host| {
            let disabled = trigger_disabled(&mut host);
            host.part(input)
                .role(Role::Button)
                .expanded(shown)
                .controls(self.content)
                .focusable(move || !disabled.get())
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
        Part::new(part_state, move |mut host| {
            let disabled = trigger_disabled(&mut host);
            let write = move |open: Option<bool>| {
                if !disabled.get_untracked() {
                    self.write(open);
                }
            };
            host.part(base)
                .node_ref(self.anchor)
                .on_press(move || write(None))
                .a11y_action(A11yAction::Toggle, move |_| write(None))
                .a11y_action(A11yAction::Expand, move |_| write(Some(true)))
                .a11y_action(A11yAction::Collapse, move |_| write(Some(false)))
        })
    }

    /// Mount `content` at the trigger while the disclosure is shown, in a
    /// fresh owner on every opening, with the content part on the overlay.
    #[track_caller]
    pub fn surface<V: IntoView>(
        self,
        placement: Placement,
        content: impl Fn() -> V + 'static,
    ) -> View {
        kivact_core::view::show(self.is_open(), move || {
            kivact_core::view::overlay(self.anchor, placement, content())
                .with_node(|node| node.part(self.content()))
        })
        .into_view()
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
