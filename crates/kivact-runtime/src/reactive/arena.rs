//! Role: Typed generational slots.
//!
//! Each App has its own arenas: graph nodes, owners and the objects owned
//! by an owner (`Runtime` in `data`). A handle is `(AppId, Key)` and holds
//! no pointer, so it is `Copy` and is checked again on every use.
//!
//! Invariants:
//! - A key `(slot, gen)` is never valid twice: the generation grows on
//!   every free and a slot whose generation saturates is retired instead of
//!   reused.
//! - A freed slot keeps a tombstone (`Tomb`) with where the entry was
//!   created and what disposed it, so a later use of the stale key reports
//!   both in its `Disposed` error.

use std::panic::Location;

/// Generational key of an arena entry (graph node, owner or object).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct Key {
    /// Slot index in the arena.
    pub(crate) slot: u32,
    /// Generation of the slot when the key was issued.
    pub(crate) generation: u32,
}

/// What is known about a freed key (for `Disposed` diagnostics).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Tomb {
    pub(crate) created_at: &'static Location<'static>,
    pub(crate) disposed_by: Option<&'static Location<'static>>,
}

struct Entry<T> {
    generation: SlotGeneration,
    state: SlotState<T>,
}

/// Generation changes only when an occupied slot is removed.
struct SlotGeneration(u32);
impl SlotGeneration {
    fn advance(&mut self) -> bool {
        if self.0 == u32::MAX {
            return false;
        }
        self.0 += 1;
        true
    }
}

enum SlotState<T> {
    Occupied(T),
    Vacant { tomb: Option<Tomb> },
    Retired { tomb: Option<Tomb> },
}

pub(crate) struct Arena<T> {
    entries: Vec<Entry<T>>,
    free: Vec<u32>,
    len: usize,
}

impl<T> Arena<T> {
    pub(crate) fn new() -> Self {
        Arena {
            entries: Vec::new(),
            free: Vec::new(),
            len: 0,
        }
    }

    pub(crate) fn insert(&mut self, value: T) -> Key {
        self.len += 1;
        if let Some(slot) = self.free.pop() {
            let e = &mut self.entries[slot as usize];
            e.state = SlotState::Occupied(value);
            return Key {
                slot,
                generation: e.generation.0,
            };
        }
        let slot = self.entries.len() as u32;
        self.entries.push(Entry {
            generation: SlotGeneration(0),
            state: SlotState::Occupied(value),
        });
        Key {
            slot,
            generation: 0,
        }
    }

    pub(crate) fn get(&self, key: Key) -> Option<&T> {
        let e = self.entries.get(key.slot as usize)?;
        if e.generation.0 == key.generation {
            match &e.state {
                SlotState::Occupied(value) => Some(value),
                _ => None,
            }
        } else {
            None
        }
    }

    pub(crate) fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        let e = self.entries.get_mut(key.slot as usize)?;
        if e.generation.0 == key.generation {
            match &mut e.state {
                SlotState::Occupied(value) => Some(value),
                _ => None,
            }
        } else {
            None
        }
    }

    pub(crate) fn contains(&self, key: Key) -> bool {
        self.get(key).is_some()
    }

    pub(crate) fn remove(&mut self, key: Key, tomb: Option<Tomb>) -> Option<T> {
        let e = self.entries.get_mut(key.slot as usize)?;
        if e.generation.0 != key.generation {
            return None;
        }
        if !matches!(e.state, SlotState::Occupied(_)) {
            return None;
        }
        let reusable = e.generation.advance();
        let next = if reusable {
            SlotState::Vacant { tomb }
        } else {
            SlotState::Retired { tomb }
        };
        let SlotState::Occupied(value) = std::mem::replace(&mut e.state, next) else {
            unreachable!("invariant: remove checked the slot was occupied");
        };
        self.len -= 1;
        if reusable {
            self.free.push(key.slot);
        }
        Some(value)
    }

    /// Tombstone of `key` if the slot has not been reused since.
    pub(crate) fn tomb(&self, key: Key) -> Option<Tomb> {
        let e = self.entries.get(key.slot as usize)?;
        if e.generation.0 != key.generation.wrapping_add(1) {
            return None;
        }
        match e.state {
            SlotState::Vacant { tomb } | SlotState::Retired { tomb } => tomb,
            SlotState::Occupied(_) => None,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    /// Allocated slots (live + free + retired).
    pub(crate) fn capacity(&self) -> usize {
        self.entries.len()
    }
}
