//! Role: Heap probe: live bytes per allocation site after mounting `ROWS` items of `MODE`.
//!
//! A counting global allocator records the call stack of every allocation
//! made after `main` starts; `SETTLE_SECS` (3) after the root mounted (the
//! window is idle) it prints the live bytes per first Kivact/Arvane function
//! and exits. Build with line
//! tables and no strip (`CARGO_PROFILE_RELEASE_STRIP=none CARGO_PROFILE_RELEASE_DEBUG=line-tables-only`) so sites resolve.
//! Modes are those of `kv_stress` (`box`, `text`, `button`, `row`).

#![allow(
    unsafe_code,
    reason = "a global allocator wrapping System is the measurement itself"
)]
#![allow(
    clippy::print_stderr,
    reason = "the probe reports its measurements on the console"
)]

use kivact::prelude::*;
use kivact_bench::env_or;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

const DEPTH: usize = 28;

struct Site {
    live: usize,
    count: usize,
    ips: [usize; DEPTH],
}

// xtask-allow(state-fields): independent: allocation telemetry of the probe; its allocator wrapper is the only writer and reader, and every combination is a valid measurement.
static PTRS: Mutex<Option<HashMap<usize, (usize, u64)>>> = Mutex::new(None);
// xtask-allow(state-fields): independent: allocation telemetry of the probe; its allocator wrapper is the only writer and reader, and every combination is a valid measurement.
static SITES: Mutex<Option<HashMap<u64, Site>>> = Mutex::new(None);
// xtask-allow(state-fields): independent: allocation telemetry of the probe; its allocator wrapper is the only writer and reader, and every combination is a valid measurement.
static TRACKING: AtomicBool = AtomicBool::new(false);
// xtask-allow(state-fields): independent: allocation telemetry of the probe; its allocator wrapper is the only writer and reader, and every combination is a valid measurement.
thread_local! { static INSIDE: Cell<bool> = const { Cell::new(false) }; }

fn record(ptr: usize, size: usize) {
    let mut ips = [0usize; DEPTH];
    let mut depth = 0;
    // SAFETY: the reentrancy guard keeps this thread out of the allocator
    // while the unsynchronized trace runs; nothing else touches `ips`.
    unsafe {
        backtrace::trace_unsynchronized(|frame| {
            ips[depth] = frame.ip() as usize;
            depth += 1;
            depth < DEPTH
        });
    }
    let key = ips[2..].iter().fold(1_469_598_103_934_665_603u64, |h, ip| {
        (h ^ *ip as u64).wrapping_mul(1_099_511_628_211)
    });
    if let Ok(mut sites) = SITES.lock() {
        let site = sites
            .get_or_insert_with(HashMap::new)
            .entry(key)
            .or_insert(Site {
                live: 0,
                count: 0,
                ips,
            });
        site.live += size;
        site.count += 1;
    }
    if let Ok(mut ptrs) = PTRS.lock() {
        ptrs.get_or_insert_with(HashMap::new)
            .insert(ptr, (size, key));
    }
}

fn forget(ptr: usize) {
    let hit = PTRS
        .lock()
        .ok()
        .and_then(|mut ptrs| ptrs.as_mut().and_then(|p| p.remove(&ptr)));
    if let Some((size, key)) = hit
        && let Ok(mut sites) = SITES.lock()
        && let Some(site) = sites.as_mut().and_then(|s| s.get_mut(&key))
    {
        site.live -= size;
        site.count -= 1;
    }
}

struct Counting;

// SAFETY: every call is forwarded to `System` unchanged; the bookkeeping
// runs outside the allocation and never reenters it (`INSIDE`).
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller's contract for `layout` is passed on as it is.
        let ptr = unsafe { System.alloc(layout) };
        if TRACKING.load(Ordering::Relaxed) && !INSIDE.with(|c| c.replace(true)) {
            record(ptr as usize, layout.size());
            INSIDE.with(|c| c.set(false));
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if !INSIDE.with(|c| c.replace(true)) {
            forget(ptr as usize);
            INSIDE.with(|c| c.set(false));
        }
        // SAFETY: `ptr` came from `System.alloc` with this `layout`.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// The first Kivact/Arvane function on the stack of `site`.
fn owner(site: &Site) -> String {
    let mut names = Vec::new();
    for ip in site.ips[2..].iter().take_while(|ip| **ip != 0) {
        backtrace::resolve(*ip as *mut std::ffi::c_void, |symbol| {
            if let Some(name) = symbol.name() {
                names.push(format!("{name:#}"));
            }
        });
    }
    names
        .iter()
        .find(|n| (n.contains("kivact") || n.contains("arvane")) && !n.contains("kivact_bench"))
        .cloned()
        .unwrap_or_else(|| names.get(3).cloned().unwrap_or_default())
}

fn dump(rows: u32) -> ! {
    INSIDE.with(|c| c.set(true));
    TRACKING.store(false, Ordering::SeqCst);
    let sites = SITES
        .lock()
        .expect("invariant: the probe never poisons its lock");
    let sites: Vec<&Site> = sites
        .iter()
        .flat_map(|s| s.values())
        .filter(|s| s.live > 0)
        .collect();
    let total: usize = sites.iter().map(|s| s.live).sum();
    let mut by_fn: HashMap<String, (usize, usize)> = HashMap::new();
    for site in sites {
        let entry = by_fn.entry(owner(site)).or_default();
        entry.0 += site.live;
        entry.1 += site.count;
    }
    let mut lines: Vec<_> = by_fn.into_iter().collect();
    lines.sort_by_key(|line| std::cmp::Reverse(line.1.0));
    eprintln!("H total_mib {:.1}", total as f64 / 1_048_576.0);
    for (name, (live, count)) in lines.iter().take(40) {
        let per_item = *live as f64 / f64::from(rows.max(1));
        eprintln!("H site {per_item:>9.1} B/item {count:>8} allocs {name}");
    }
    std::process::exit(0)
}

fn main() -> kivact::Result<()> {
    let rows: u32 = env_or("ROWS", 3000);
    let mode: String = env_or("MODE", "box".to_owned());
    let settle: u64 = env_or("SETTLE_SECS", 3);
    TRACKING.store(true, Ordering::SeqCst);
    App::new("kv_heap").run(move || {
        let mode = mode.clone();
        // Measure the finished screen: the timer starts once the root has
        // mounted (the first frame follows within the settle time).
        on_mount(move || {
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(settle));
                dump(rows);
            });
        });
        column((
            text("Header"),
            scroll(column(for_index(
                (0..rows).collect::<Vec<u32>>(),
                move |i, _| match mode.as_str() {
                    "text" => text(format!("Row {i}")).into_view(),
                    "button" => button("x").into_view(),
                    "row" => row((text(format!("Row {i}")), button("x"))).into_view(),
                    _ => node().style(style().height(20).width(200)).into_view(),
                },
            ))),
        ))
    })
}
