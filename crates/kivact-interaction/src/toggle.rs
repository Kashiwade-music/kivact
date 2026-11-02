//! Role: Button, check box, switch and toggle button semantics without appearance.
//!
//! [`use_button`] is the regular way to make an element a momentary button:
//! the part adds the role and the focus, and the caller's `on_press` is the
//! activation. [`use_checkbox`], [`use_switch`] and [`use_toggle_button`]
//! edit one `Binding<bool>` (react-aria's `useCheckbox`, `useSwitch` and
//! `useToggleButton`); attaching the control adds the role, `checked`, the
//! focus, the input and the assistive technology actions. The protocol has
//! no pressed state, so a toggle button publishes `checked` too, and no
//! On/Off/Mixed value text is added. Only a check box has a mixed state, and
//! the caller's writer clears it. A disabled control takes no input or
//! assistive technology action and leaves the tab order.

use kivact_core::primitive::{Part, PartState, part};
use kivact_core::view::{
    A11yAction, CheckedState, Handled, IntoValue, Key, KeyEvent, Modifiers, Role,
};
use kivact_core::{Binding, Value};

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
    disabled: Option<Value<bool>>,
}

/// Create a momentary button without creating a view. Setup only.
///
/// ```no_run
/// use kivact_core::control::node;
/// use kivact_core::control::NodeBuilder;
/// use kivact_interaction::use_button;
/// let control = use_button().control();
/// let view = node().part(control).name("Save").on_press(|| {});
/// ```
#[track_caller]
pub fn use_button() -> ButtonApi {
    ButtonApi { disabled: None }
}

impl ButtonApi {
    /// Ignore input and assistive technology while true.
    ///
    /// Also marks the root disabled and makes it unfocusable. Setup only.
    #[track_caller]
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = Some(disabled.into_value());
        self
    }

    /// Create the semantic and input attachment. Read its state before moving it.
    #[track_caller]
    pub fn control(self) -> Part<PartState> {
        let control = part(|input| input).role(Role::Button).focusable(true);
        match self.disabled {
            Some(disabled) => control.disabled(disabled),
            None => control,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/// Copy handle for a control that flips one `Binding<bool>`: a check box
/// ([`use_checkbox`]), a switch ([`use_switch`]) or a toggle button
/// ([`use_toggle_button`]). Attach its [`control`](Self::control) to the
/// caller's element with `.part()`.
///
/// Every kind flips on Space, a pointer press, Activate and Toggle. A toggle
/// button also flips on Enter; a check box and a switch consume plain Enter
/// without flipping. Caller key handlers run first.
#[derive(Clone, Copy)]
pub struct Toggle {
    checked: Binding<bool>,
    kind: Kind,
    indeterminate: Option<Value<bool>>,
    disabled: Option<Value<bool>>,
}

fn toggle(checked: Binding<bool>, kind: Kind) -> Toggle {
    Toggle {
        checked,
        kind,
        indeterminate: None,
        disabled: None,
    }
}

/// A check box (`Role::CheckBox`) editing `checked`, react-aria's
/// `useCheckbox`. It may publish a mixed state with
/// [`indeterminate`](Toggle::indeterminate). Setup only.
///
/// ```no_run
/// use kivact_core::{signal, control::node};
/// use kivact_core::control::NodeBuilder;
/// use kivact_interaction::use_checkbox;
/// let checked = signal(false);
/// let control = use_checkbox(checked).control();
/// let state = control.state();
/// let view = node().part(control).name("Remember me");
/// ```
#[track_caller]
pub fn use_checkbox(checked: impl Into<Binding<bool>>) -> Toggle {
    toggle(checked.into(), Kind::Checkbox)
}

/// A switch (`Role::Switch`) editing `checked`, react-aria's `useSwitch`.
/// Setup only.
#[track_caller]
pub fn use_switch(checked: impl Into<Binding<bool>>) -> Toggle {
    toggle(checked.into(), Kind::Switch)
}

/// A toggle button (`Role::Button` with a checked state, ARIA's
/// `aria-pressed`) editing `pressed`, react-aria's `useToggleButton`.
/// Setup only.
#[track_caller]
pub fn use_toggle_button(pressed: impl Into<Binding<bool>>) -> Toggle {
    toggle(pressed.into(), Kind::Button)
}

impl Toggle {
    /// Publish a mixed (partially checked) state to assistive technology while
    /// true. Only a check box has a mixed state; a switch and a toggle button
    /// ignore it.
    ///
    /// The checked value stays the binary value of the binding. Pressing a
    /// mixed check box writes `true` to the checked binding, whatever its
    /// current value. The toggle never writes the indeterminate value, which the
    /// caller owns: clear it in the binding's writer (a [`binding`](kivact_core::binding))
    /// so the control leaves the mixed state when
    /// it is pressed. A `Signal` binding does not notify when it already holds
    /// `true`. Setup only.
    #[track_caller]
    pub fn indeterminate(mut self, mixed: impl IntoValue<bool>) -> Self {
        self.indeterminate = Some(mixed.into_value());
        self
    }

    /// Ignore input and assistive technology while true.
    ///
    /// Also marks the root disabled and makes it unfocusable. Setup only.
    #[track_caller]
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = Some(disabled.into_value());
        self
    }

    fn is_disabled(self) -> bool {
        self.disabled.is_some_and(|value| value.get_untracked())
    }

    fn mixed(self) -> Option<Value<bool>> {
        self.indeterminate.filter(|_| self.kind == Kind::Checkbox)
    }

    fn press(self) {
        if !self.is_disabled() {
            let mixed = self.mixed().is_some_and(|value| value.get());
            self.checked.set(mixed || !self.checked.get());
        }
    }

    /// Create the semantic and input attachment. Read its state before moving it.
    #[track_caller]
    pub fn control(self) -> Part<ToggleState> {
        let checked = self.checked;
        let mixed = self.mixed();
        let mut control = part(move |input| ToggleState {
            input,
            checked: if mixed.is_some_and(|value| value.get()) {
                CheckedState::Indeterminate
            } else {
                CheckedState::from_bool(checked.get())
            },
        })
        .role(match self.kind {
            Kind::Checkbox => Role::CheckBox,
            Kind::Switch => Role::Switch,
            Kind::Button => Role::Button,
        })
        .focusable(true)
        .on_press(move || self.press())
        .checked(checked)
        .on_a11y(A11yAction::Toggle, move || self.press());
        if let Some(disabled) = self.disabled {
            control = control.disabled(disabled);
        }
        if let Some(mixed) = mixed {
            control = control.indeterminate(mixed);
        }
        if self.kind != Kind::Button {
            control = control.on_key(|event: KeyEvent| {
                if event.key == Key::Enter && event.modifiers == Modifiers::default() {
                    Handled::Yes
                } else {
                    Handled::No
                }
            });
        }
        control
    }
}
