//! Role: Handle keys, wheel and pointer actions on a numeric control.

use kivact::view::{
    A11yEvent, A11yRequest, ButtonPress, Handled, Key, KeyEvent, MouseButton, PointerEvent,
    PointerPhase, WheelEvent,
};

use super::builder::NumericKind;
use super::machine;
use super::view::NumericView;

fn sane_factor(factor: f64) -> f64 {
    if factor.is_finite() && factor > 0.0 {
        factor
    } else {
        1.0
    }
}

impl NumericView {
    pub(super) fn key(&self, e: KeyEvent) -> Handled {
        let range = self.range;
        let reset = self.reset;
        let mixed = self.mixed;
        let mode = self.mode;
        let typing = self.typing;
        let scalar = self.scalar;
        let pointer = self.pointer;
        let text_entry = self.text_entry;
        let text_open = move || matches!(mode.get(), machine::State::Typing);
        let set = move |value: f64| {
            scalar.once(|| scalar.set(value));
        };

        let adjusts = matches!(
            e.key(),
            Key::ArrowRight
                | Key::ArrowUp
                | Key::ArrowLeft
                | Key::ArrowDown
                | Key::PageUp
                | Key::PageDown
                | Key::Home
                | Key::End
                | Key::Backspace
        );
        if text_open() {
            // The text field runs first. Consume the keys it leaves
            // unhandled at the ends of its text, so that none of them
            // writes the value.
            return if adjusts { Handled::Yes } else { Handled::No };
        }
        if e.key() == Key::Enter && !text_entry {
            return Handled::No;
        }
        // A scrub owns the value until it ends, and a mixed value
        // takes only typed text, which Enter opens.
        if (adjusts || e.key() == Key::Enter) && (pointer.is_active() || (mixed.get() && adjusts)) {
            return Handled::Yes;
        }
        let factor = self.scrub_policy().factor(e.modifiers());
        match e.key() {
            Key::Enter if e.repeat() => {}
            Key::Enter => typing.show(()),
            Key::ArrowRight | Key::ArrowUp => {
                scalar.once(|| scalar.step_by(factor));
            }
            Key::ArrowLeft | Key::ArrowDown => {
                scalar.once(|| scalar.step_by(-factor));
            }
            Key::PageUp => {
                scalar.once(|| scalar.step_by(10.0 * factor));
            }
            Key::PageDown => {
                scalar.once(|| scalar.step_by(-10.0 * factor));
            }
            Key::Home => set(range.get_untracked().min()),
            Key::End => set(range.get_untracked().max()),
            Key::Backspace => set(reset.get_untracked()),
            _ => return Handled::No,
        }
        Handled::Yes
    }

    pub(super) fn wheel(&self, e: WheelEvent) -> Handled {
        let mixed = self.mixed;
        let mode = self.mode;
        let scalar = self.scalar;
        let pointer = self.pointer;
        let text_open = move || matches!(mode.get(), machine::State::Typing);

        if text_open() || mixed.get() {
            return Handled::No;
        }
        if pointer.is_active() {
            return Handled::Yes;
        }
        let delta = if e.delta().y != 0.0 {
            e.delta().y
        } else {
            e.delta().x
        };
        if delta == 0.0 {
            return Handled::No;
        }
        // A wheel notch points up (positive y) and increases the value.
        let signed = -delta;
        let policy = self.scrub_policy();
        let notch = policy.wheel_notch;
        let measured = notch.is_finite() && notch > 0.0;
        let rest = self.wheel_rest.get();
        // A reversed scroll starts a new accumulation.
        let rest = if rest * signed < 0.0 { 0.0 } else { rest };
        let rest = rest + signed;
        let notches = if measured {
            (rest / notch).trunc()
        } else {
            signed.signum()
        };
        if notches != 0.0 {
            let used = if measured { notches * notch } else { rest };
            self.wheel_rest.set(rest - used);
            let fine = if policy.fine.is_some_and(|key| key.is_held(e.modifiers())) {
                sane_factor(policy.fine_factor)
            } else {
                1.0
            };
            scalar.once(|| scalar.step_by(notches * fine));
        } else {
            // A sub-notch event writes nothing and reports no edit.
            self.wheel_rest.set(rest);
        }
        Handled::Yes
    }

    pub(super) fn pointer(&self, e: PointerEvent) -> Handled {
        let kind = self.kind;
        let reset = self.reset;
        let mixed = self.mixed;
        let mode = self.mode;
        let typing = self.typing;
        let scalar = self.scalar;
        let pointer = self.pointer;
        let text_entry = self.text_entry;
        let text_open = move || matches!(mode.get(), machine::State::Typing);
        let set = move |value: f64| {
            scalar.once(|| scalar.set(value));
        };

        if text_open() {
            return Handled::Yes;
        }
        if mixed.get() {
            if matches!(
                e.phase(),
                PointerPhase::Down(ButtonPress {
                    button: MouseButton::Primary,
                    ..
                })
            ) && text_entry
                && matches!(kind, NumericKind::Number | NumericKind::Slider)
            {
                typing.show(());
            }
            return Handled::Yes;
        }
        let reset_click = self
            .scrub_policy()
            .reset_click
            .is_some_and(|key| key.is_held(e.modifiers()));
        if let PointerPhase::Down(press) = e.phase()
            && press.button == MouseButton::Primary
            && (press.clicks >= 2 || reset_click)
        {
            pointer.cancel();
            set(reset.get_untracked());
            return Handled::Yes;
        }
        Handled::No
    }

    pub(super) fn action(&self, event: A11yEvent) {
        let mixed = self.mixed;
        let scalar = self.scalar;
        let pointer = self.pointer;
        // Only an action that will write is an edit: a text that is not a
        // number and does not parse reports nothing.
        let target = match &event.request {
            A11yRequest::SetValue(text) => self.set_value_target(text),
            _ => None,
        };
        let writes = event.region.is_none()
            && !pointer.is_active()
            && match &event.request {
                A11yRequest::SetValue(_) => target.is_some(),
                _ => !mixed.get_untracked(),
            };
        if writes {
            scalar.once(|| match &event.request {
                A11yRequest::Increment => scalar.step_by(1.0),
                A11yRequest::Decrement => scalar.step_by(-1.0),
                A11yRequest::SetValue(_) => target.is_some_and(|value| scalar.set(value)),
                _ => false,
            });
        }
    }

    /// The value an assistive `SetValue` writes: a finite plain number as is,
    /// otherwise what the field's parser reads from the text (from the current
    /// value). `None` writes nothing.
    fn set_value_target(&self, text: &str) -> Option<f64> {
        text.parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .or_else(|| {
                (self.parse)(text, self.value.get_untracked())
                    .ok()
                    .filter(|value| value.is_finite())
            })
    }
}
