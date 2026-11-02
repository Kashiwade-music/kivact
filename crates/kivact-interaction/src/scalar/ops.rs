//! Role: The writes of a scalar value: normalize, step and set through the binding.

use crate::policy::scalar::ScalarRange;
use kivact_core::view::{A11yEvent, A11yRequest, Key};
use kivact_core::{Binding, Value};

/// What one key press does to the value.
#[derive(Clone, Copy)]
pub(crate) enum KeyOp {
    Steps(f64),
    To(f64),
}

/// A value's binding and range: the writes every scalar-like behavior shares
/// (a slider thumb, a number field, a splitter).
#[derive(Clone, Copy)]
pub(crate) struct ScalarOps {
    pub(crate) binding: Binding<f64>,
    pub(crate) range: Value<ScalarRange>,
}

impl ScalarOps {
    /// Normalize by the current range and invoke the binding writer. `false`
    /// rejects a non-finite input without writing.
    #[track_caller]
    pub(crate) fn set(self, value: f64) -> bool {
        let Some(value) = self.range.get_untracked().normalize(value) else {
            return false;
        };
        self.binding.set(value);
        true
    }

    /// Change the current normalized value by `key_step` units, `steps` times.
    #[track_caller]
    pub(crate) fn step_by(self, steps: f64, key_step: f64) -> bool {
        if !steps.is_finite() {
            return false;
        }
        let range = self.range.get_untracked();
        let current = self.binding.get_untracked();
        let amount = steps * key_step;
        let next = if amount.is_finite() {
            range.offset(current, amount)
        } else {
            range.normalize(current).and_then(|_| {
                range.normalize(if steps > 0.0 {
                    range.max()
                } else {
                    range.min()
                })
            })
        };
        let Some(next) = next else { return false };
        self.binding.set(next);
        true
    }

    /// Directional increments, ten steps for PageUp/PageDown, and Home/End;
    /// `mirror` swaps ArrowLeft and ArrowRight.
    pub(crate) fn key_op(self, key: Key, mirror: bool) -> Option<KeyOp> {
        let range = self.range.get();
        Some(match key {
            Key::ArrowRight if mirror => KeyOp::Steps(-1.0),
            Key::ArrowLeft if mirror => KeyOp::Steps(1.0),
            Key::ArrowRight | Key::ArrowUp => KeyOp::Steps(1.0),
            Key::ArrowLeft | Key::ArrowDown => KeyOp::Steps(-1.0),
            Key::PageUp => KeyOp::Steps(10.0),
            Key::PageDown => KeyOp::Steps(-10.0),
            Key::Home => KeyOp::To(range.min()),
            Key::End => KeyOp::To(range.max()),
            _ => return None,
        })
    }

    #[track_caller]
    pub(crate) fn apply(self, op: KeyOp, key_step: f64) -> bool {
        match op {
            KeyOp::Steps(steps) => self.step_by(steps, key_step),
            KeyOp::To(value) => self.set(value),
        }
    }

    /// Increment, Decrement and SetValue through the same writer. Returns
    /// whether the writer was offered a value; an invalid or missing value is not.
    #[track_caller]
    pub(crate) fn a11y(self, event: &A11yEvent, key_step: f64) -> bool {
        match &event.request {
            A11yRequest::Increment => self.step_by(1.0, key_step),
            A11yRequest::Decrement => self.step_by(-1.0, key_step),
            A11yRequest::SetValue(v) => v.parse::<f64>().ok().is_some_and(|v| self.set(v)),
            _ => false,
        }
    }
}
