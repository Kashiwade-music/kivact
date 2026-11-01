//! Role: Tests for performance and scale measurements.
//! (100k keyed data). Run in release:
//! `cargo test -p kivact-public-api-tests --release --test e2e -- scale:: --ignored --nocapture --test-threads=1`

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::support::mount;
use kivact::prelude::*;
use kivact::test::Screen;

static CLONES: AtomicUsize = AtomicUsize::new(0);
static KEYS: AtomicUsize = AtomicUsize::new(0);
static ROW_RUNS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, PartialEq)]
struct Row {
    id: u64,
    label: String,
}

impl Clone for Row {
    fn clone(&self) -> Self {
        CLONES.fetch_add(1, Ordering::Relaxed);
        Row {
            id: self.id,
            label: self.label.clone(),
        }
    }
}

fn rows(n: u64) -> Vec<Row> {
    (0..n)
        .map(|id| Row {
            id,
            label: format!("row {id}"),
        })
        .collect()
}

fn median(mut v: Vec<u64>) -> u64 {
    v.sort_unstable();
    v[v.len() / 2]
}

#[test]
#[ignore = "release-mode measurement"]
fn one_text_change_is_independent_of_node_count() {
    let mut results = Vec::new();
    for &n in &[200usize, 2_000, 10_000, 50_000] {
        let (screen, sigs) = mount(move || {
            let sigs: Vec<Signal<String>> = (0..n).map(|i| signal(format!("t{i}"))).collect();
            let views: Vec<_> = sigs.iter().map(|s| text(*s)).collect();
            (sigs, column(views))
        });
        let target = sigs[n / 2];
        let mut times = Vec::new();
        let mut ops = Vec::new();
        let mut touched = Vec::new();
        for i in 0..301u32 {
            let before = screen.stats();
            target.set(format!("changed {i}"));
            screen.run_until_idle();
            let after = screen.stats();
            if i > 0 {
                times.push(after.core_ns - before.core_ns);
                ops.push(after.ops - before.ops);
                touched.push(after.host_touched_last as u64);
            }
        }
        let t = median(times);
        let o = median(ops);
        let h = median(touched);
        println!("granularity n={n:>6} core_median_ns={t:>7} ops_per_change={o} host_touched={h}");
        assert_eq!(o, 1, "one SetProps op per text change");
        assert_eq!(h, 1, "one host node touched");
        results.push((n, t));
    }
    let t10k = results[2].1 as f64;
    let t50k = results[3].1 as f64;
    let ratio = t50k / t10k;
    println!("granularity 50k/10k core time ratio = {ratio:.3}");
    // One-sided and coarse: a cost proportional to the node count makes
    // 5x the nodes about 5x slower, while timing noise on a busy machine
    // moves either side by tens of percent (and 50k is often the faster).
    assert!(
        ratio <= 2.0 || t50k - t10k < 2_000.0,
        "core time depends on node count: {ratio:.3}"
    );
}

struct Measure {
    ns: u64,
    clones: usize,
    keys: usize,
    rows: usize,
    ops: u64,
}

fn measure(screen: &Screen, f: impl FnOnce()) -> Measure {
    CLONES.store(0, Ordering::Relaxed);
    KEYS.store(0, Ordering::Relaxed);
    ROW_RUNS.store(0, Ordering::Relaxed);
    let before = screen.stats();
    f();
    screen.run_until_idle();
    let after = screen.stats();
    Measure {
        ns: after.core_ns - before.core_ns,
        clones: CLONES.load(Ordering::Relaxed),
        keys: KEYS.load(Ordering::Relaxed),
        rows: ROW_RUNS.load(Ordering::Relaxed),
        ops: after.ops - before.ops,
    }
}

fn list_app(n: u64) -> (Screen, Signal<Vec<Row>>) {
    mount(move || {
        let data = signal(rows(n));
        (
            data,
            column(for_each(
                data,
                |r| {
                    KEYS.fetch_add(1, Ordering::Relaxed);
                    r.id
                },
                |_, r| {
                    text(move || {
                        ROW_RUNS.fetch_add(1, Ordering::Relaxed);
                        r.with(|r| r.label.clone())
                    })
                },
            )),
        )
    })
}

#[test]
#[ignore = "release-mode measurement"]
fn keyed_rows_scale_to_100k() {
    let mut table: Vec<(String, u64, u64)> = Vec::new();
    for &n in &[10_000u64, 100_000] {
        let (screen, data) = list_app(n);
        let m = measure(&screen, || {
            data.update(|v| v[(n / 2) as usize].label = "edited".into())
        });
        println!(
            "scale n={n} edit1: core_ns={} item_clones={} key_calls={} row_effect_runs={} ops={}",
            m.ns, m.clones, m.keys, m.rows, m.ops
        );
        assert_eq!(m.clones, 1, "only the edited item is cloned");
        assert_eq!(m.rows, 1, "only the edited row re-renders");
        assert_eq!(m.ops, 1);
        assert_eq!(m.keys as u64, n, "one O(N) key scan");
        let reorder = measure(&screen, || data.update(|v| v.reverse()));
        let retain = measure(&screen, || data.update(|v| v.retain(|r| r.id % 2 == 0)));
        let (mscreen, mdata) = list_app(n);
        let mount1k = measure(&mscreen, || {
            mdata.update(|v| {
                v.extend((n..n + 1000).map(|id| Row {
                    id,
                    label: format!("row {id}"),
                }))
            })
        });
        let clear = measure(&screen, || data.set(Vec::new()));
        println!(
            "scale n={n} reorder_ns={} (ops {}) retain_ns={} (ops {}) mount1k_ns={} (ops {}) clear_ns={} (ops {})",
            reorder.ns,
            reorder.ops,
            retain.ns,
            retain.ops,
            mount1k.ns,
            mount1k.ops,
            clear.ns,
            clear.ops
        );
        table.push(("reorder".into(), n, reorder.ns));
        table.push(("retain".into(), n, retain.ns));
        table.push(("mount1k".into(), n, mount1k.ns));
        table.push(("clear".into(), n, clear.ns));
        table.push(("edit1".into(), n, m.ns));
    }
    // dependency 20k: a memo reading N signals, one of which changes
    for &n in &[2_000usize, 20_000] {
        let (screen, sigs) = mount(move || {
            let sigs: Vec<Signal<u64>> = (0..n).map(|i| signal(i as u64)).collect();
            let all = sigs.clone();
            let sum = memo(move || all.iter().map(|s| s.get()).sum::<u64>());
            (sigs, text(move || sum.get().to_string()))
        });
        let m = measure(&screen, || sigs[n / 2].update(|v| *v += 1));
        println!("scale deps={n} recompute_ns={} ops={}", m.ns, m.ops);
        table.push(("deps".into(), n as u64 * 5, m.ns));
    }
    for op in ["reorder", "retain", "mount1k", "clear", "edit1", "deps"] {
        let v: Vec<&(String, u64, u64)> = table.iter().filter(|r| r.0 == op).collect();
        let ratio = v[1].2 as f64 / v[0].2.max(1) as f64;
        println!("scale ratio 10x input {op}: {ratio:.2}");
        if op != "mount1k" {
            assert!(ratio < 35.0, "{op}: 10x input took {ratio:.1}x time");
        }
    }
}
