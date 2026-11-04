//! Role: Normalizes platform modifier samples into changes.

use kivact_types::Modifiers;

use crate::events::ShellEvent;

/// The current normalized keyboard modifiers.
#[derive(Clone, Copy, Debug, Default)]
pub struct ModifierState(Modifiers);

impl ModifierState {
    /// Update the current sample, emitting an event only when it changes.
    pub fn update(&mut self, modifiers: Modifiers) -> Option<ShellEvent> {
        if self.0 == modifiers {
            return None;
        }
        self.0 = modifiers;
        Some(ShellEvent::Modifiers(modifiers))
    }

    /// The most recent normalized sample.
    pub fn current(&self) -> Modifiers {
        self.0
    }
}
