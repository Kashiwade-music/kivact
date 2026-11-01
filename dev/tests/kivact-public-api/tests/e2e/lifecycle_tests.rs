//! Role: Tests for keyed list lifecycle and row stability.
//!
//! Keyed-list delete / reinsert / reorder / filter / clear, driven by
//! random probes (from the adversarial design survey): rows keep their
//! owners by key, stale row handles never connect to a new row, and
//! nothing is left after a clear.

#![allow(
    clippy::type_complexity,
    reason = "the probes stash handles of their setup in shared cells"
)]

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use kivact::diagnostics::ErrorKind;
use kivact::prelude::*;
use kivact::test::{Screen, render};

use crate::support::random::{Rng, residue, seeds, texts};

// ---------------------------------------------------------------------------
// Keyed list: delete key → reinsert same key; reorder → filter → clear.

type RowLog = Rc<RefCell<Vec<(u32, u32, Value<(u32, u32)>)>>>;

fn list_screen() -> (Screen, Signal<Vec<(u32, u32)>>, RowLog) {
    let log: RowLog = Rc::new(RefCell::new(Vec::new()));
    let l2 = log.clone();
    let counter = Rc::new(Cell::new(0u32));
    let slot: Rc<Cell<Option<Signal<Vec<(u32, u32)>>>>> = Rc::new(Cell::new(None));
    let s2 = slot.clone();
    let screen = render(move || {
        let items = signal(Vec::<(u32, u32)>::new());
        s2.set(Some(items));
        column((
            text("head"),
            for_each(
                items,
                |it: &(u32, u32)| it.0,
                move |k, v| {
                    let id = counter.get() + 1;
                    counter.set(id);
                    l2.borrow_mut().push((id, k, v));
                    let local = signal(0u8);
                    row((
                        text(move || format!("{}:{}#{}", k, v.get().1, id)),
                        show(
                            move || v.get().1 % 3 == 0,
                            move || text(format!("fizz#{id}")),
                        ),
                        {
                            let label = ("x").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .on_press(move || local.update(|n| *n += 1)),
                    ))
                },
            ),
            text("tail"),
        ))
    });
    let items = slot.get().expect("setup ran");
    (screen, items, log)
}

fn dedup(v: &[(u32, u32)]) -> Vec<(u32, u32)> {
    let mut seen = HashSet::new();
    v.iter().filter(|(k, _)| seen.insert(*k)).copied().collect()
}

fn shuffle(rng: &mut Rng, v: &mut [(u32, u32)]) {
    for i in (1..v.len()).rev() {
        let j = rng.below(i + 1);
        v.swap(i, j);
    }
}

fn list_step(
    rng: &mut Rng,
    cur: &[(u32, u32)],
    removed: &mut Vec<u32>,
    next_key: &mut u32,
) -> (&'static str, Vec<(u32, u32)>) {
    let mut v = cur.to_vec();
    let op = rng.below(11);
    let name = match op {
        0 if !v.is_empty() => {
            let i = rng.below(v.len());
            removed.push(v.remove(i).0);
            "remove"
        }
        1 if !removed.is_empty() => {
            let k = removed.remove(rng.below(removed.len()));
            let at = rng.below(v.len() + 1);
            v.insert(at, (k, rng.below(10) as u32));
            "reinsert same key"
        }
        2 if !v.is_empty() => {
            let i = rng.below(v.len());
            v[i].1 = rng.below(10) as u32;
            "update value"
        }
        3 => {
            shuffle(rng, &mut v);
            "reorder"
        }
        4 => {
            v.reverse();
            "reverse"
        }
        5 => {
            let keep: Vec<bool> = v.iter().map(|_| rng.chance(60)).collect();
            let mut i = 0;
            v.retain(|it| {
                let k = keep[i];
                i += 1;
                if !k {
                    removed.push(it.0);
                }
                k
            });
            "filter"
        }
        6 if rng.chance(30) => {
            removed.extend(v.iter().map(|it| it.0));
            v.clear();
            "clear"
        }
        7 if !v.is_empty() => {
            let i = rng.below(v.len());
            let dup = (v[i].0, 99);
            let at = rng.below(v.len() + 1);
            v.insert(at, dup);
            "duplicate key"
        }
        8 => "same list",
        _ => {
            for _ in 0..=rng.below(4) {
                *next_key += 1;
                let at = rng.below(v.len() + 1);
                v.insert(at, (*next_key, rng.below(10) as u32));
            }
            "insert new"
        }
    };
    (name, v)
}

#[test]
fn keyed_list_delete_reinsert_reorder_filter_clear() {
    for seed in seeds() {
        let mut rng = Rng::new(seed);
        let (screen, items, log) = list_screen();
        let baseline = residue(&screen);
        let mut cur: Vec<(u32, u32)> = Vec::new(); // raw (may contain dups)
        let mut removed = Vec::new();
        let mut next_key = 0u32;
        let mut mounted: HashMap<u32, u32> = HashMap::new(); // key -> mount id
        let mut max_id = 0u32;
        for step in 0..60 {
            let (name, next) = list_step(&mut rng, &dedup(&cur), &mut removed, &mut next_key);
            let batches = screen.stats().batches;
            items.set(next.clone());
            screen.run_until_idle();
            let ctx = format!("seed {seed} step {step} ({name})");
            let expect = dedup(&next);
            if dedup(&cur) == expect && name == "same list" {
                assert_eq!(
                    screen.stats().batches,
                    batches,
                    "{ctx}: equal list sent a batch"
                );
            }
            // rendered order and values
            let texts: Vec<String> = texts(&screen)
                .into_iter()
                .filter(|t| t.contains(':'))
                .collect();
            assert_eq!(texts.len(), expect.len(), "{ctx}: {texts:?} vs {expect:?}");
            let mut new_mounted = HashMap::new();
            for (t, (k, v)) in texts.iter().zip(&expect) {
                let (kv, id) = t.split_once('#').expect("row text");
                assert_eq!(kv, format!("{k}:{v}"), "{ctx}");
                let id: u32 = id.parse().expect("id");
                match mounted.get(k) {
                    Some(old) => assert_eq!(*old, id, "{ctx}: key {k} lost its owner"),
                    None => assert!(id > max_id, "{ctx}: key {k} reused mount id {id}"),
                }
                new_mounted.insert(*k, id);
            }
            max_id = max_id.max(new_mounted.values().copied().max().unwrap_or(0));
            // stale row handles never connect to a new row
            for (id, k, v) in log.borrow().iter() {
                let live = new_mounted.get(k) == Some(id);
                match v.try_get() {
                    Some(val) => {
                        assert!(live, "{ctx}: disposed row #{id} still reads {val:?}");
                        assert_eq!(expect.iter().find(|it| it.0 == *k), Some(&val), "{ctx}");
                    }
                    None => assert!(!live, "{ctx}: live row #{id} reads None"),
                }
            }
            let dups = next.len() - expect.len();
            if dups > 0 {
                let dk = screen
                    .errors()
                    .iter()
                    .filter(|e| matches!(e.kind(), ErrorKind::DuplicateKey { .. }))
                    .count();
                assert!(dk > 0, "{ctx}: duplicate key not reported");
            }
            mounted = new_mounted;
            cur = next;
        }
        items.set(Vec::new());
        screen.run_until_idle();
        assert_eq!(
            residue(&screen),
            baseline,
            "seed {seed}: residue after clear"
        );
    }
}
