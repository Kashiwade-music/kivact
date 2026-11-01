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
    generation: u32,
    value: Option<T>,
    tomb: Option<Tomb>,
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
            e.value = Some(value);
            e.tomb = None;
            return Key {
                slot,
                generation: e.generation,
            };
        }
        let slot = self.entries.len() as u32;
        self.entries.push(Entry {
            generation: 0,
            value: Some(value),
            tomb: None,
        });
        Key {
            slot,
            generation: 0,
        }
    }

    pub(crate) fn get(&self, key: Key) -> Option<&T> {
        let e = self.entries.get(key.slot as usize)?;
        if e.generation == key.generation {
            e.value.as_ref()
        } else {
            None
        }
    }

    pub(crate) fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        let e = self.entries.get_mut(key.slot as usize)?;
        if e.generation == key.generation {
            e.value.as_mut()
        } else {
            None
        }
    }

    pub(crate) fn contains(&self, key: Key) -> bool {
        self.get(key).is_some()
    }

    pub(crate) fn remove(&mut self, key: Key, tomb: Option<Tomb>) -> Option<T> {
        let e = self.entries.get_mut(key.slot as usize)?;
        if e.generation != key.generation {
            return None;
        }
        let value = e.value.take()?;
        self.len -= 1;
        e.tomb = tomb;
        if e.generation < u32::MAX {
            e.generation += 1;
            self.free.push(key.slot);
        }
        Some(value)
    }

    /// Tombstone of `key` if the slot has not been reused since.
    pub(crate) fn tomb(&self, key: Key) -> Option<Tomb> {
        let e = self.entries.get(key.slot as usize)?;
        if e.value.is_none() && e.generation == key.generation.wrapping_add(1) {
            e.tomb
        } else {
            None
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
