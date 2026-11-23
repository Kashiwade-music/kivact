//! Role: Mutable state of a virtual list: rows, heights, seek and focus.

use std::collections::HashMap;

use kivact_runtime::{Owner, Signal};
use kivact_types::NodeId;

use super::heights::Heights;

pub(super) struct VRow<T: 'static> {
    pub(super) owner: Owner,
    pub(super) wrapper: NodeId,
    pub(super) sig: Signal<T>,
    pub(super) top: f64,
}

/// What the host reported of the list's scroll box.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Viewport {
    pub(super) offset: f64,
    pub(super) height: f64,
}

/// The viewport height the visible range assumes until the host reports one.
pub(super) const UNREPORTED_HEIGHT: f64 = 600.0;

/// The `scroll_to` request in progress.
pub(super) enum Seek<K> {
    Idle,
    /// The row `key` is to be scrolled into view, until the host has done it.
    Pending {
        key: K,
        issued: Option<Issued>,
        /// How many scrolls were asked for in this request.
        steps: u8,
    },
}

/// The last scroll the list asked for in a request: the offset it was at and the one it asked for.
#[derive(Clone, Copy)]
pub(super) struct Issued {
    pub(super) from: f64,
    pub(super) to: f64,
}

pub(super) struct VState<K: 'static, T: 'static> {
    pub(super) rows: HashMap<K, VRow<T>>,
    pub(super) measured: HashMap<K, f64>,
    /// Row heights by list index (`Measured` lists), rebuilt when the list changes.
    pub(super) heights: Heights,
    /// The list index of each key (its first occurrence), rebuilt when the list changes.
    pub(super) index_of: HashMap<K, usize>,
    /// How many items `index_of` was built for.
    pub(super) indexed: usize,
    /// The `scroll_to` request in progress.
    pub(super) seek: Seek<K>,
    /// The row that holds the keyboard focus (`keep_focused`).
    pub(super) held: HeldState<K>,
}

/// Rows mounted at once above which an unconstrained viewport is reported.
pub(super) const UNCONSTRAINED_ROWS: usize = 100;

pub(super) struct HeldState<K> {
    pub(super) value: Option<K>,
    pub(super) changed: kivact_runtime::Trigger,
}
impl<K: PartialEq> HeldState<K> {
    pub(super) fn key(&self) -> Option<&K> {
        self.value.as_ref()
    }
    pub(super) fn set(&mut self, key: Option<K>) {
        if self.value != key {
            self.value = key;
            self.changed.notify();
        }
    }
}
