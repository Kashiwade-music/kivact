//! Role: A cache whose entries live for the frames that use them, within a cost budget.
//!
//! The renderer keeps pixels from one frame to the next (converted images,
//! pixels made from a scene's values, pattern atlas slots), so a scene that
//! repeats costs no conversion. [`FrameCache::frame`] is the only way to use a
//! cache: its closure is one frame, and when the closure returns every entry
//! the frame did not use is dropped. Nothing depends on a caller bracketing a
//! frame correctly.
//!
//! Invariants:
//! - The kept entries cost no more than `max_cost` in all. A value beyond
//!   that is returned for its frame and not kept.
//! - A value whose key is the address of an allocation carries that
//!   allocation in a [`Pinned`], so the address is not reused by other
//!   pixels while the entry lives.
//! - Thread: owned by its renderer, used on the renderer's thread.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

/// Values by key, kept while the frames use them (see the module invariants).
pub(crate) struct FrameCache<K, V> {
    entries: HashMap<K, Entry<V>>,
    /// The number of the last frame begun (0 before the first).
    frame: u64,
    /// The sum of the costs of `entries`.
    cost: usize,
    /// The most cost the entries may add up to.
    max_cost: usize,
}

struct Entry<V> {
    value: V,
    cost: usize,
    /// The last frame that used it.
    frame: u64,
}

impl<K: Hash + Eq, V: Clone> FrameCache<K, V> {
    /// An empty cache whose entries may cost `max_cost` in all.
    pub(crate) fn new(max_cost: usize) -> Self {
        Self {
            entries: HashMap::new(),
            frame: 0,
            cost: 0,
            max_cost,
        }
    }

    /// Draws one frame: `body` uses the cache through the scope it is given,
    /// and when it returns, the entries the frame did not use are dropped.
    pub(crate) fn frame<R>(&mut self, body: impl FnOnce(FrameScope<'_, K, V>) -> R) -> R {
        self.frame_evicting(body, |_, _| {})
    }

    /// [`FrameCache::frame`], which also hands each entry it drops, with its
    /// key and value, to `on_evict` once `body` has returned.
    pub(crate) fn frame_evicting<R>(
        &mut self,
        body: impl FnOnce(FrameScope<'_, K, V>) -> R,
        mut on_evict: impl FnMut(K, V),
    ) -> R {
        self.frame += 1;
        let result = body(FrameScope { cache: &mut *self });
        self.drop_unused(&mut on_evict);
        result
    }

    /// Drops the entries the frame did not use, and their cost.
    fn drop_unused(&mut self, on_evict: &mut impl FnMut(K, V)) {
        let frame = self.frame;
        let mut freed = 0;
        for (key, entry) in self.entries.extract_if(|_, entry| entry.frame != frame) {
            freed += entry.cost;
            on_evict(key, entry.value);
        }
        self.cost -= freed;
    }
}

/// The cache as the frame being drawn uses it.
pub(crate) struct FrameScope<'a, K, V> {
    cache: &'a mut FrameCache<K, V>,
}

impl<K: Hash + Eq, V: Clone> FrameScope<'_, K, V> {
    /// The value under `key`, which this frame uses. On a miss, `make` gives
    /// the value and its cost. The value is kept while the budget allows,
    /// and it is returned either way.
    pub(crate) fn get_or_insert_with(&mut self, key: K, make: impl FnOnce() -> (V, usize)) -> V {
        if let Some(value) = self.hit(&key) {
            return value;
        }
        let (value, cost) = make();
        self.keep(key, value.clone(), cost);
        value
    }

    /// The value under `key`, which this frame uses. On a miss, `make` is
    /// called and its value is kept, at `cost`, only while it fits the
    /// budget; when it does not fit, `make` is not called and the result is
    /// `None`.
    #[cfg(any(feature = "gpu", test))]
    pub(crate) fn get_or_insert_if_room(
        &mut self,
        key: K,
        cost: usize,
        make: impl FnOnce() -> V,
    ) -> Option<V> {
        if let Some(value) = self.hit(&key) {
            return Some(value);
        }
        if self.cache.cost + cost > self.cache.max_cost {
            return None;
        }
        let value = make();
        self.keep(key, value.clone(), cost);
        Some(value)
    }

    /// The value under `key` if the cache has it, marked as used by this frame.
    fn hit(&mut self, key: &K) -> Option<V> {
        let frame = self.cache.frame;
        let entry = self.cache.entries.get_mut(key)?;
        entry.frame = frame;
        Some(entry.value.clone())
    }

    /// Keeps `value` under `key` at `cost`, when the budget allows.
    fn keep(&mut self, key: K, value: V, cost: usize) {
        let cache = &mut *self.cache;
        if cache.cost + cost <= cache.max_cost {
            cache.cost += cost;
            cache.entries.insert(
                key,
                Entry {
                    value,
                    cost,
                    frame: cache.frame,
                },
            );
        }
    }
}

/// A cached value that keeps an allocation alive. Its key names the
/// allocation by address; holding the `Arc` stops another allocation from
/// taking that address while the entry lives.
#[derive(Clone)]
pub(crate) struct Pinned<V> {
    /// The cached value.
    pub(crate) value: V,
    /// The allocation the key names, if any. Held, never read.
    _pin: Option<Arc<[u8]>>,
}

impl<V> Pinned<V> {
    /// A value whose key names no allocation.
    pub(crate) fn unpinned(value: V) -> Self {
        Self { value, _pin: None }
    }

    /// A value whose key names `pin`.
    pub(crate) fn new(value: V, pin: Arc<[u8]>) -> Self {
        Self {
            value,
            _pin: Some(pin),
        }
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the frame cache's keeping, dropping, eviction and budget rules.

    use super::FrameCache;

    /// Uses `key` in its own frame. A miss costs `cost`. Returns whether it
    /// missed.
    fn use_key(cache: &mut FrameCache<u8, u32>, key: u8, cost: usize) -> bool {
        cache.frame(|mut frame| {
            let mut missed = false;
            frame.get_or_insert_with(key, || {
                missed = true;
                (u32::from(key), cost)
            });
            missed
        })
    }

    #[test]
    fn entry_survives_the_next_frame_only_if_used_again() {
        let mut cache = FrameCache::new(100);
        assert!(use_key(&mut cache, 1, 1), "the first frame inserts it");
        assert!(!use_key(&mut cache, 1, 1), "used in the next frame: a hit");
        cache.frame(|_| {});
        assert!(cache.entries.is_empty(), "not used in a frame: dropped");
        assert!(use_key(&mut cache, 1, 1), "dropped entries miss again");
    }

    #[test]
    fn unused_entries_are_dropped_when_the_frame_ends() {
        let mut cache = FrameCache::new(100);
        cache.frame(|mut frame| {
            frame.get_or_insert_with(1, || (1, 3));
            frame.get_or_insert_with(2, || (2, 5));
        });
        cache.frame(|mut frame| {
            frame.get_or_insert_with(1, || (1, 3));
        });
        assert_eq!(cache.entries.len(), 1);
        assert!(cache.entries.contains_key(&1));
        assert_eq!(cache.cost, 3);
    }

    #[test]
    fn cost_budget_is_respected() {
        let mut cache = FrameCache::new(10);
        cache.frame(|mut frame| {
            frame.get_or_insert_with(1, || (1, 6));
            frame.get_or_insert_with(2, || (2, 6));
        });
        assert_eq!(cache.cost, 6, "the second entry would exceed the budget");
        assert!(!cache.entries.contains_key(&2));
        let (missed_1, missed_2) = cache.frame(|mut frame| {
            let mut missed_1 = false;
            let mut missed_2 = false;
            frame.get_or_insert_with(1, || {
                missed_1 = true;
                (1, 6)
            });
            frame.get_or_insert_with(2, || {
                missed_2 = true;
                (2, 6)
            });
            (missed_1, missed_2)
        });
        assert!(!missed_1, "the kept entry still hits");
        assert!(missed_2, "a value not kept misses again");
        assert_eq!(cache.cost, 6);
    }

    #[test]
    fn value_over_budget_is_returned_but_not_kept() {
        let mut cache = FrameCache::new(10);
        let value = cache.frame(|mut frame| frame.get_or_insert_with(1, || (7, 11)));
        assert_eq!(value, 7);
        assert!(cache.entries.is_empty());
        assert_eq!(cache.cost, 0);
    }

    #[test]
    fn cost_returns_to_zero_when_everything_is_evicted() {
        let mut cache = FrameCache::new(10);
        cache.frame(|mut frame| {
            for key in 1_u8..=3 {
                frame.get_or_insert_with(key, || (u32::from(key), 3));
            }
        });
        assert_eq!(cache.cost, 9);
        cache.frame(|_| {});
        assert!(cache.entries.is_empty());
        assert_eq!(cache.cost, 0);
        cache.frame(|mut frame| {
            frame.get_or_insert_with(9, || (9, 10));
        });
        assert_eq!(cache.cost, 10, "the whole budget is free again");
    }

    #[test]
    fn room_check_does_not_make_a_value_over_budget() {
        let mut cache = FrameCache::new(10);
        let (kept, refused) = cache.frame(|mut frame| {
            let kept = frame.get_or_insert_if_room(1, 6, || 1_u32);
            let mut made = false;
            let refused = frame.get_or_insert_if_room(2, 6, || {
                made = true;
                2
            });
            assert!(!made, "make is not called when the cost does not fit");
            (kept, refused)
        });
        assert_eq!(kept, Some(1));
        assert_eq!(refused, None);
        assert_eq!(cache.cost, 6);
    }

    #[test]
    fn room_check_returns_a_kept_value_without_the_budget_check() {
        let mut cache = FrameCache::new(10);
        cache.frame(|mut frame| frame.get_or_insert_if_room(1, 10, || 1_u32));
        let value = cache.frame(|mut frame| {
            frame.get_or_insert_if_room(1, 100, || panic!("a kept value is not made again"))
        });
        assert_eq!(value, Some(1));
    }

    #[test]
    fn eviction_hands_exactly_the_dropped_entries_to_the_callback() {
        let mut cache = FrameCache::new(100);
        cache.frame(|mut frame| {
            for key in 1_u8..=3 {
                frame.get_or_insert_with(key, || (u32::from(key) * 10, 1));
            }
        });
        let mut evicted = Vec::new();
        cache.frame_evicting(
            |mut frame| {
                frame.get_or_insert_with(2, || (20, 1));
            },
            |key, value| evicted.push((key, value)),
        );
        evicted.sort_unstable();
        assert_eq!(evicted, vec![(1, 10), (3, 30)]);
        assert_eq!(cache.entries.len(), 1);
        assert_eq!(cache.cost, 1);
    }
}
