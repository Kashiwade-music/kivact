//! Role: Button, check box, switch and toggle button semantics without appearance.
//!
//! [`use_button`] is the regular way to make an element a momentary button:
//! the part adds the role and the focus, and the caller's `on_press` is the
//! activation. [`use_checkbox`], [`use_switch`] and [`use_toggle_button`]
//! edit one `Binding<bool>` (react-aria's `useCheckbox`, `useSwitch` and
//! `useToggleButton`); attaching the control adds the role, `checked`, the
//! focus, the input and the assistive technology actions. The protocol has
//! no pressed state, so a toggle button publishes `checked` too, and no
//! On/Off/Mixed value text is added. Only a check box ([`CheckboxApi`]) has a
//! mixed state, and the caller's writer clears it. A disabled control takes no input or
//! assistive technology action and leaves the tab order.

use crate::options_cell::OptionsCell;
use kivact_core::custom::{Part, PartState, part};
use kivact_core::state::IntoValue;
use kivact_core::state::{Binding, Signal, Value, signal};
use kivact_core::view::{A11yAction, CheckedState, Handled, Key, KeyEvent, Modifiers, Role};

/// Copy handle for a momentary button, the behavior of react-aria's
/// `useButton`. Attach its [`control`](Self::control) to the caller's
/// element with `.part()`.
///
/// This is the way to make an element a button: the part sets
/// `Role::Button` and focusability, and the element's `on_press` runs on a
/// pointer click, Enter, Space and the Activate action. Writing
/// `node().role(Role::Button).focusable(true)` by hand only adds the
/// semantics, a lower layer for an element whose activation is someone
/// else's (a canvas region, a native control).
#[derive(Clone, Copy)]
pub struct ButtonApi {
    observed: Signal<Option<Value<PartState>>>,
    disabled: Signal<Option<Value<bool>>>,
}

/// Create a momentary button without creating a view. Setup only.
///
/// ```no_run
/// use kivact_core::view::node;
/// use kivact_core::view::NodeBuilder;
/// use kivact_interaction::use_button;
/// let control = use_button().control();
/// let view = node().part(control).name("Save").on_press(|| {});
/// ```
#[track_caller]
pub fn use_button() -> ButtonApi {
    ButtonApi {
        observed: signal(None),
        disabled: signal(None),
    }
}

impl ButtonApi {
    /// Drawing state of the actual mounted control.
    pub fn control_state(self) -> Value<PartState> {
        kivact_core::state::memo(move || {
            self.observed
                .get()
                .map(|state| state.get())
                .unwrap_or_default()
        })
    }

    /// Ignore input and assistive technology while true.
    ///
    /// Also marks the root disabled and makes it unfocusable. Setup only.
    #[track_caller]
    pub fn disabled(self, disabled: impl IntoValue<bool>) -> Self {
        let disabled = disabled.into_value();
        self.disabled.update(|slot| *slot = Some(disabled));
        self
    }

    /// Create the semantic and input attachment. Read its state before moving it.
    #[track_caller]
    pub fn control(self) -> Part<PartState> {
        let disabled = self.disabled;
        let input = part(|input| input);
        let part_state = input.state();
        self.observed.set(Some(part_state));
        Part::new(part_state, move |mut host| {
            let caller_disabled = host.accessible_disabled();

            host.part(input)
                .role(Role::Button)
                .focusable(true)
                .disabled(move || {
                    caller_disabled.is_some_and(|value| value.get())
                        || disabled.with(|d| d.is_some_and(|d| d.get()))
                })
        })
    }
}

/// What a toggle is; the mixed state belongs to the check box.
#[derive(Clone, Copy)]
enum Kind {
    Checkbox,
    Switch,
    Button,
}

/// Reactive drawing state from the actual attached input target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToggleState {
    pub input: PartState,
    pub checked: CheckedState,
}

/// What every toggle shares: the binding, the kind and the disabled value.
#[derive(Clone, Copy)]
struct Core {
    // xtask-allow(state-fields): The checked binding is caller-owned input, independent of mount observation.
    checked: Binding<bool>,
    kind: Kind,
    // xtask-allow(state-fields): EditSession already owns the edit lifecycle machine; bool is its payload type.
    edit: crate::EditSession<bool>,
    observed: Signal<Option<Value<ToggleState>>>,
    options: OptionsCell<ToggleOptions>,
    // xtask-allow(state-fields): Host disabled is a mounted node input, not a lifecycle phase.
    host_disabled: Signal<Option<Value<bool>>>,
}

/// Copy handle for a check box ([`use_checkbox`]), react-aria's
/// `useCheckbox`. Attach its [`control`](Self::control) to the caller's
/// element with `.part()`.
///
/// It flips on Space, a pointer press, Activate and Toggle, and consumes plain
/// Enter without flipping. It may publish a mixed state with
/// [`indeterminate`](Self::indeterminate). Caller key handlers run first.
#[derive(Clone, Copy)]
pub struct CheckboxApi {
    core: Core,
}

/// Copy handle for a control that flips one `Binding<bool>`: a switch
/// ([`use_switch`]) or a toggle button ([`use_toggle_button`]). Attach its
/// [`control`](Self::control) to the caller's element with `.part()`.
///
/// Both flip on Space, a pointer press, Activate and Toggle. A toggle button
/// also flips on Enter; a switch consumes plain Enter without flipping. Caller
/// key handlers run first. (A check box is a [`CheckboxApi`].)
#[derive(Clone, Copy)]
pub struct SwitchApi {
    core: Core,
}

#[derive(Clone, Copy)]
pub struct ToggleButtonApi {
    core: Core,
}

fn core(checked: Binding<bool>, kind: Kind) -> Core {
    Core {
        checked,
        kind,
        edit: crate::EditSession::new(move |value| checked.set(value)),
        observed: signal(None),
        options: OptionsCell::new(ToggleOptions::default()),
        host_disabled: signal(None),
    }
}

#[derive(Clone, Default)]
struct ToggleOptions {
    disabled: Option<Value<bool>>,
    indeterminate: Option<Value<bool>>,
}

/// A check box (`Role::CheckBox`) editing `checked`, react-aria's
/// `useCheckbox`. It may publish a mixed state with
/// [`indeterminate`](CheckboxApi::indeterminate). Setup only.
///
/// ```no_run
/// use kivact_core::state::{signal};
/// use kivact_core::{view::node};
/// use kivact_core::view::NodeBuilder;
/// use kivact_interaction::use_checkbox;
/// let checked = signal(false);
/// let control = use_checkbox(checked).control();
/// let state = control.state();
/// let view = node().part(control).name("Remember me");
/// ```
#[track_caller]
pub fn use_checkbox(checked: impl kivact_core::state::IntoBinding<bool>) -> CheckboxApi {
    CheckboxApi {
        core: core(checked.into_binding(), Kind::Checkbox),
    }
}

/// A switch (`Role::Switch`) editing `checked`, react-aria's `useSwitch`.
/// Setup only.
#[track_caller]
pub fn use_switch(checked: impl kivact_core::state::IntoBinding<bool>) -> SwitchApi {
    SwitchApi {
        core: core(checked.into_binding(), Kind::Switch),
    }
}

/// A toggle button (`Role::Button` with a checked state, ARIA's
/// `aria-pressed`) editing `pressed`, react-aria's `useToggleButton`.
/// Setup only.
#[track_caller]
pub fn use_toggle_button(pressed: impl kivact_core::state::IntoBinding<bool>) -> ToggleButtonApi {
    ToggleButtonApi {
        core: core(pressed.into_binding(), Kind::Button),
    }
}

impl CheckboxApi {
    /// Observe a discrete toggle edit.
    pub fn on_edit<R: kivact_core::view::HandlerResult>(
        self,
        f: impl Fn(kivact_core::view::EditPhase) -> R + 'static,
    ) -> Self {
        self.core.edit.on_edit(f);
        self
    }

    /// Drawing state of the mounted control.
    pub fn control_state(self) -> Value<ToggleState> {
        self.core.control_state()
    }
    /// Publish a mixed (partially checked) state to assistive technology while
    /// true.
    ///
    /// The checked value stays the binary value of the binding. Pressing a
    /// mixed check box writes `true` to the checked binding, whatever its
    /// current value. The check box never writes the indeterminate value, which
    /// the caller owns: clear it in the binding's writer (a
    /// [`binding`](kivact_core::binding)) so the control leaves the mixed state
    /// when it is pressed. A `Signal` binding does not notify when it already
    /// holds `true`. Setup only.
    #[track_caller]
    pub fn indeterminate(self, mixed: impl IntoValue<bool>) -> Self {
        let mixed = mixed.into_value();
        self.core
            .options
            .update(|options| options.indeterminate = Some(mixed));
        self
    }

    /// Ignore input and assistive technology while true.
    ///
    /// Also marks the root disabled and makes it unfocusable. Setup only.
    #[track_caller]
    pub fn disabled(self, disabled: impl IntoValue<bool>) -> Self {
        self.core.set_disabled(disabled.into_value());
        self
    }

    /// Create the semantic and input attachment. Read its state before moving it.
    #[track_caller]
    pub fn control(self) -> Part<ToggleState> {
        self.core.control()
    }
}

impl SwitchApi {
    /// Observe a discrete toggle edit.
    pub fn on_edit<R: kivact_core::view::HandlerResult>(
        self,
        f: impl Fn(kivact_core::view::EditPhase) -> R + 'static,
    ) -> Self {
        self.core.edit.on_edit(f);
        self
    }

    /// Drawing state of the mounted control.
    pub fn control_state(self) -> Value<ToggleState> {
        self.core.control_state()
    }
    /// Ignore input and assistive technology while true.
    ///
    /// Also marks the root disabled and makes it unfocusable. Setup only.
    #[track_caller]
    pub fn disabled(self, disabled: impl IntoValue<bool>) -> Self {
        self.core.set_disabled(disabled.into_value());
        self
    }

    /// Create the semantic and input attachment. Read its state before moving it.
    #[track_caller]
    pub fn control(self) -> Part<ToggleState> {
        self.core.control()
    }
}
impl ToggleButtonApi {
    /// Observe a discrete toggle edit.
    pub fn on_edit<R: kivact_core::view::HandlerResult>(
        self,
        f: impl Fn(kivact_core::view::EditPhase) -> R + 'static,
    ) -> Self {
        self.core.edit.on_edit(f);
        self
    }

    /// Drawing state of the mounted control.
    pub fn control_state(self) -> Value<ToggleState> {
        self.core.control_state()
    }
    /// Ignore input and assistive technology while true.
    ///
    /// Also marks the root disabled and makes it unfocusable. Setup only.
    #[track_caller]
    pub fn disabled(self, disabled: impl IntoValue<bool>) -> Self {
        self.core.set_disabled(disabled.into_value());
        self
    }

    /// Create the semantic and input attachment. Read its state before moving it.
    #[track_caller]
    pub fn control(self) -> Part<ToggleState> {
        self.core.control()
    }
}

impl Core {
    fn control_state(self) -> Value<ToggleState> {
        kivact_core::state::memo(move || {
            self.observed
                .get()
                .map(|state| state.get())
                .unwrap_or(ToggleState {
                    input: PartState::default(),
                    checked: CheckedState::from_bool(self.checked.get()),
                })
        })
    }
    fn set_disabled(self, disabled: Value<bool>) {
        self.options
            .update(|options| options.disabled = Some(disabled));
    }

    fn is_disabled(self) -> bool {
        self.options
            .get()
            .disabled
            .is_some_and(|value| value.get_untracked())
            || self
                .host_disabled
                .with_untracked(|value| value.is_some_and(|value| value.get_untracked()))
    }

    /// The mixed value is meaningful only for a checkbox.
    fn is_mixed(self) -> bool {
        matches!(self.kind, Kind::Checkbox)
            && self
                .options
                .value()
                .get()
                .indeterminate
                .is_some_and(|value| value.get())
    }

    fn press(self) {
        if !self.is_disabled() {
            let mixed = self.is_mixed();
            self.edit.once(self.checked.get_untracked(), || {
                self.checked.set(mixed || !self.checked.get())
            });
        }
    }

    #[track_caller]
    fn control(self) -> Part<ToggleState> {
        let checked = self.checked;
        let mixed = matches!(self.kind, Kind::Checkbox);
        let options = self.options.value();
        let is_mixed = move || self.is_mixed();
        let input = part(move |input| ToggleState {
            input,
            checked: if is_mixed() {
                CheckedState::Indeterminate
            } else {
                CheckedState::from_bool(checked.get())
            },
        });
        let part_state = input.state();
        self.observed.set(Some(part_state));
        Part::new(part_state, move |mut host| {
            let caller_disabled = host.accessible_disabled();
            self.host_disabled.set(caller_disabled);

            let mut control = host
                .part(input)
                .role(match self.kind {
                    Kind::Checkbox => Role::CheckBox,
                    Kind::Switch => Role::Switch,
                    Kind::Button => Role::Button,
                })
                .focusable(true)
                .on_press(move || self.press())
                .checked(checked)
                .a11y_action(A11yAction::Toggle, move |_| self.press())
                .disabled(move || {
                    caller_disabled.is_some_and(|value| value.get())
                        || options.get().disabled.is_some_and(|value| value.get())
                });
            if mixed {
                control = control.indeterminate(is_mixed);
            }
            if !matches!(self.kind, Kind::Button) {
                control = control.on_key(|event: KeyEvent| {
                    if event.key() == Key::Enter && event.modifiers() == Modifiers::default() {
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                });
            }
            control
        })
    }
}
