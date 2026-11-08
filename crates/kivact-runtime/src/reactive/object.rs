//! Role: Owner-bound objects and their lifecycle management.
//!
//! Owner-bound objects (the state of Var, Callback, NodeRef, Task,
//! Resource, Action, FrameLoop, Animated and Selector): data registered
//! under an owner and disposed with it.
//!
//! Invariants:
//! - An object is listed in its owner's `objects` at `owner_index`
//!   (registration and removal are paired; a removal moves the last entry
//!   into the hole and fixes its index).
//! - `on_dispose` runs once, before the data is dropped, when the object is
//!   removed or its owner is disposed.
//! - A removed object leaves a tombstone, so a later use reports where it
//!   was created (`Disposed`).

use std::any::Any;
use std::panic::Location;
use std::rc::Rc;

use crate::reactive::arena;
use crate::reactive::arena::Key;
use crate::reactive::data::{Object, Runtime};
use crate::reactive::error::{ErrorKind, misuse_at};

impl Runtime {
    /// Register an object of `owner`, disposed with it (`on_dispose` runs
    /// first); on a gone owner it is released at once.
    pub(crate) fn insert_object(
        &self,
        owner: Key,
        data: Rc<dyn Any>,
        name: &'static str,
        on_dispose: Option<Box<dyn FnOnce(&Runtime)>>,
        at: &'static Location<'static>,
    ) -> Key {
        let key = self.objects.borrow_mut().insert(Object {
            owner,
            owner_index: 0,
            data,
            name,
            on_dispose,
            created_at: at,
        });
        let index = self.owners.borrow_mut().get_mut(owner).map(|o| {
            o.objects.push(key);
            o.objects.len() as u32 - 1
        });
        match index {
            Some(i) => {
                if let Some(obj) = self.objects.borrow_mut().get_mut(key) {
                    obj.owner_index = i;
                }
            }
            // A gone owner: released at once (the key is a tombstone).
            None => self.release_orphan_object(key, owner),
        }
        key
    }

    /// The data of object `key` as `T`, if it is alive and has that type.
    pub(crate) fn object<T: 'static>(&self, key: Key) -> Option<Rc<T>> {
        let data = self.objects.borrow().get(key).map(|o| o.data.clone())?;
        data.downcast::<T>().ok()
    }

    /// The owner of object `key`, if it is alive.
    pub(crate) fn object_owner(&self, key: Key) -> Option<Key> {
        self.objects.borrow().get(key).map(|o| o.owner)
    }

    /// Object data or a located `Disposed` panic.
    pub(crate) fn object_or_panic<T: 'static>(
        &self,
        key: Key,
        what: &str,
        at: &'static Location<'static>,
    ) -> Rc<T> {
        match self.object::<T>(key) {
            Some(d) => d,
            None => {
                let tomb = self.objects.borrow().tomb(key);
                misuse_at(
                    ErrorKind::Disposed {
                        created_at: tomb.map(|t| t.created_at),
                        disposed_by: tomb.and_then(|t| t.disposed_by),
                    },
                    format!("{what} used after its owner was disposed"),
                    at,
                )
            }
        }
    }

    /// Located `Disposed` misuse for a handle whose owner is gone.
    pub(crate) fn owner_disposed_panic(
        &self,
        owner: Key,
        what: &str,
        at: &'static Location<'static>,
    ) -> ! {
        let tomb = self.owners.borrow().tomb(owner);
        misuse_at(
            ErrorKind::Disposed {
                created_at: tomb.map(|t| t.created_at),
                disposed_by: tomb.and_then(|t| t.disposed_by),
            },
            format!("{what} used after its owner was disposed"),
            at,
        )
    }

    /// Located `Disposed` misuse unless `owner` is alive (platform requests and environment reads).
    pub(crate) fn require_owner(&self, owner: Key, what: &str, at: &'static Location<'static>) {
        if !self.owner_alive(owner) {
            self.owner_disposed_panic(owner, what, at);
        }
    }

    /// Remove one object before its owner is disposed (finished task etc.).
    pub(crate) fn remove_object(&self, key: Key) {
        let removed = {
            let mut objects = self.objects.borrow_mut();
            let Some(obj) = objects.get(key) else { return };
            let (owner, index, at) = (obj.owner, obj.owner_index as usize, obj.created_at);
            let moved = {
                let mut owners = self.owners.borrow_mut();
                owners.get_mut(owner).and_then(|o| {
                    if index < o.objects.len() && o.objects[index] == key {
                        o.objects.swap_remove(index);
                        o.objects.get(index).copied()
                    } else {
                        None
                    }
                })
            };
            if let Some(m) = moved.and_then(|m| objects.get_mut(m)) {
                m.owner_index = index as u32;
            }
            objects.remove(
                key,
                Some(arena::Tomb {
                    created_at: at,
                    disposed_by: None,
                }),
            )
        };
        if let Some(mut obj) = removed {
            if let Some(f) = obj.on_dispose.take() {
                f(self);
            }
            drop(obj);
        }
    }
}
