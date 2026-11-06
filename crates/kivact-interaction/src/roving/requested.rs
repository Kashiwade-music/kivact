//! Role: Preserve the caller binding as the authoritative requested key.
use super::machine;
use crate::options_cell::OptionsCell;
use kivact_core::state::{Binding, MachineHandle, machine};

#[derive(Clone)]
enum Source<K: kivact_core::state::ItemKey> {
    Local,
    Shared(Binding<Option<K>>),
}

pub(crate) struct Requested<K: kivact_core::state::ItemKey> {
    source: OptionsCell<Source<K>>,
    pub(super) machine: MachineHandle<machine::Machine<K>>,
}
impl<K: kivact_core::state::ItemKey> Clone for Requested<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: kivact_core::state::ItemKey> Copy for Requested<K> {}
impl<K: kivact_core::state::ItemKey> Requested<K> {
    pub(super) fn new() -> Self {
        let source = OptionsCell::new(Source::Local);
        let machine = machine(
            machine::State {
                roving: machine::Roving::Unset,
                restore: machine::Restore::Nothing,
            },
            move || machine::Machine::new(matches!(source.get(), Source::Shared(_))),
            move |_, machine::Effect::Request(key)| {
                if let Source::Shared(binding) = source.get() {
                    binding.set(key);
                }
            },
        );
        Self { source, machine }
    }
    pub(crate) fn bind(self, binding: Binding<Option<K>>) {
        self.source.update(|slot| *slot = Source::Shared(binding));
        self.machine.send(machine::Event::Share);
    }
    pub(crate) fn get(self) -> Option<K> {
        match self.source.value().get() {
            Source::Local => match self.machine.state().get().roving {
                machine::Roving::Requested { key, .. } => Some(key),
                _ => None,
            },
            Source::Shared(binding) => binding.get(),
        }
    }
    pub(crate) fn get_untracked(self) -> Option<K> {
        kivact_core::state::untrack(|| self.get())
    }
    pub(crate) fn set(self, key: Option<K>) {
        self.machine.send(machine::Event::Request(key));
    }
}

/// Removal reports enter the same focus machine as requests and observations.
pub(crate) struct Restore<K: kivact_core::state::ItemKey>(MachineHandle<machine::Machine<K>>);
impl<K: kivact_core::state::ItemKey> Clone for Restore<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: kivact_core::state::ItemKey> Copy for Restore<K> {}
impl<K: kivact_core::state::ItemKey> Restore<K> {
    pub(super) fn new(machine: MachineHandle<machine::Machine<K>>) -> Self {
        Self(machine)
    }
    pub(crate) fn set(self, token: Option<kivact_core::custom::NodeToken>) {
        self.0.send(machine::Event::Restore(token));
    }
    pub(super) fn take(self) -> Option<kivact_core::custom::NodeToken> {
        let restore = self.0.state().get_untracked().restore;
        self.set(None);
        match restore {
            machine::Restore::Nothing => None,
            machine::Restore::From(token) => Some(token),
        }
    }
}
