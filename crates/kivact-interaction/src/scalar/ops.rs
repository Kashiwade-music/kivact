//! Role: The writes of a scalar value: normalize, step and set through the binding.

use crate::options_cell::OptionsCell;
use crate::policy::scalar::{ScalarRange, ScalarStep};
use crate::policy::scalar_mapping::ScalarMapping;
use kivact_core::state::{Binding, Value};
use kivact_core::view::{A11yEvent, A11yRequest, Key};

/// What one key press does to the value.
#[derive(Clone, Copy)]
pub(crate) enum KeyOp {
    Steps(f64),
    To(f64),
}

/// A value's binding, range and mapping: the writes every scalar-like behavior
/// shares (a slider thumb, a number field, a splitter).
#[derive(Clone, Copy)]
pub(crate) struct ScalarOps {
    pub(crate) binding: Binding<f64>,
    pub(crate) range: Value<ScalarRange>,
    pub(crate) mapping: OptionsCell<ScalarMapping>,
}

impl ScalarOps {
    /// A scalar with the linear mapping. Setup only.
    #[track_caller]
    pub(crate) fn new(binding: Binding<f64>, range: Value<ScalarRange>) -> Self {
        Self {
            binding,
            range,
            mapping: OptionsCell::new(ScalarMapping::linear()),
        }
    }

    /// Normalize by the current range and invoke the binding writer. `false`
    /// rejects a non-finite input without writing.
    #[track_caller]
    pub(crate) fn set(self, value: f64) -> bool {
        let Some(value) = self.range.get_untracked().normalize(value) else {
            return false;
        };
        self.update(value)
    }

    /// Write the value at a position along the control, through the mapping.
    /// `false` writes nothing for a non-finite position or curve value.
    #[track_caller]
    pub(crate) fn set_position(self, position: f64) -> bool {
        if !position.is_finite() {
            return false;
        }
        let range = self.range.get_untracked();
        let Some(value) = self.mapping.get().value(position, range) else {
            return false;
        };
        self.set(value)
    }

    /// Change the current normalized value by `key_step` units, `steps` times.
    #[track_caller]
    pub(crate) fn step_by(self, steps: f64, key_step: impl Into<ScalarStep>) -> bool {
        if !steps.is_finite() {
            return false;
        }
        match key_step.into() {
            ScalarStep::Value(step) => self.step_value(steps, step),
            ScalarStep::Position(step) => self.step_position(steps, step),
        }
    }

    /// A step in the value's units, saturating at the bounds.
    #[track_caller]
    fn step_value(self, steps: f64, key_step: f64) -> bool {
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
        match next {
            Some(next) => self.update(next),
            None => false,
        }
    }

    /// A step in travel, through the mapping; a step that quantization rounds
    /// back to the current value moves one quantization step.
    #[track_caller]
    fn step_position(self, steps: f64, step: f64) -> bool {
        let range = self.range.get_untracked();
        let mapping = self.mapping.get();
        let current = self.binding.get_untracked();
        let travel = steps * step;
        let from = mapping.position(current, range);
        let target = from + travel;
        let target = if target.is_finite() {
            target
        } else if travel > 0.0 {
            1.0
        } else {
            0.0
        };
        let Some(value) = mapping.value(target, range) else {
            return false;
        };
        if value != current {
            return self.update(value);
        }
        if travel == 0.0 {
            return false;
        }
        let Some(quantization) = range.step() else {
            return false;
        };
        match range.offset(current, quantization.copysign(steps)) {
            Some(moved) if moved != current => self.update(moved),
            _ => false,
        }
    }

    /// The binding writer after this scalar's checks. A projected binding may
    /// still reject the edit.
    #[track_caller]
    fn update(self, value: f64) -> bool {
        self.binding.set(value);
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
    pub(crate) fn apply(self, op: KeyOp, key_step: ScalarStep) -> bool {
        match op {
            KeyOp::Steps(steps) => self.step_by(steps, key_step),
            KeyOp::To(value) => self.set(value),
        }
    }

    /// Increment, Decrement and SetValue through the same writer. Returns
    /// whether the writer was offered a value; an invalid or missing value is not.
    #[track_caller]
    pub(crate) fn a11y(self, event: &A11yEvent, key_step: impl Into<ScalarStep>) -> bool {
        let key_step = key_step.into();
        match &event.request {
            A11yRequest::Increment => self.step_by(1.0, key_step),
            A11yRequest::Decrement => self.step_by(-1.0, key_step),
            A11yRequest::SetValue(v) => v.parse::<f64>().ok().is_some_and(|v| self.set(v)),
            _ => false,
        }
    }
}
