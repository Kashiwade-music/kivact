//! Role: Memory churn benchmark scenarios.
//!
//! Mount and single-item update cost at 1k, 10k and 50k nodes: a scroll box of rows
//! (`Box` + `Text`) is shown at once, then one visible row's text changes
//! 30 times. Reported: change → first presented frame for the mount and
//! for each update, and the host's time of that frame. The `relayout`
//! variant changes the row's height (20 or 30 px) with its text, so every
//! update lays the list out again and moves the rows below it.

use kivact::view::View;
use std::cell::{Cell, RefCell};

use kivact::prelude::*;

use crate::report::{Json, frame_log, mark, ms, stats, unix_ns};
use crate::scenes::{SETTLE, every_frame};

const UPDATES: u64 = 30;
/// Ticks between updates (each update gets its own frame).
const EVERY: u64 = 3;

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Settle,
    Mounting,
    MountSettle(u64),
    Updating(u64),
    Done,
}

pub(crate) fn churn(nodes: usize, relayout: bool) -> View {
    let name = if relayout {
        format!("relayout_{}k", nodes / 1000)
    } else {
        format!("churn_{}k", nodes / 1000)
    };
    // Root, column, status text, the scroll box, then a Box + Text per row.
    let rows = (nodes - 4) / 2;
    let shown = signal(false);
    let version = signal(0u64);
    let items: Vec<u32> = (0..rows as u32).collect();
    let list = move || {
        let items = items.clone();
        scroll(for_index(items, move |i, _| {
            let label = if i == 5 {
                text(move || format!("row 5 v{}", version.get()))
            } else {
                text(format!("row {i}"))
            };
            let row = node().child(label);
            if i == 5 && relayout {
                row.style(move || {
                    style()
                        .height(20 + 10 * (version.get() % 2) as i32)
                        .shrink(0)
                })
            } else {
                row.style(style().height(20).shrink(0))
            }
        }))
        .style(style().grow(1).min_height(0))
    };
    let phase = Cell::new(Phase::Settle);
    let mount = Cell::new((0u128, 0u128));
    let changes = RefCell::new(Vec::new());
    let update_begin = Cell::new(0u128);
    let name_ = name.clone();
    every_frame(move |tick, _| {
        match phase.get() {
            Phase::Settle if tick >= SETTLE => {
                let t = mark("mount_begin");
                mount.set((t, 0));
                shown.set(true);
                phase.set(Phase::Mounting);
            }
            Phase::Mounting => {
                if let Some(f) = frame_log().first_after(mount.get().0) {
                    mark("mount_end");
                    mount.set((mount.get().0, f.end));
                    phase.set(Phase::MountSettle(tick));
                } else if !frame_log().enabled {
                    phase.set(Phase::MountSettle(tick));
                }
            }
            Phase::MountSettle(since) if tick >= since + SETTLE => {
                update_begin.set(mark("update_begin"));
                phase.set(Phase::Updating(0));
            }
            Phase::Updating(k) if k < UPDATES * EVERY => {
                if k % EVERY == 0 {
                    changes.borrow_mut().push(unix_ns());
                    version.set(k / EVERY + 1);
                }
                phase.set(Phase::Updating(k + 1));
            }
            Phase::Updating(_) => {
                let end = mark("update_end");
                let log = frame_log();
                let (t0, shown_at) = mount.get();
                let mount_frame = log.first_after(t0);
                let mut latency = Vec::new();
                let mut work = Vec::new();
                for &t in changes.borrow().iter() {
                    if let Some(f) = log.first_after(t) {
                        latency.push(ms(f.end - t));
                        work.push(f.work as f64 / 1e6);
                    }
                }
                Json::new(&name_)
                    .int("nodes", nodes as u128)
                    .num(
                        "mount_ms",
                        if shown_at > t0 {
                            ms(shown_at - t0)
                        } else {
                            f64::NAN
                        },
                    )
                    .num(
                        "mount_work_ms",
                        mount_frame.map_or(f64::NAN, |f| f.work as f64 / 1e6),
                    )
                    .int("updates", changes.borrow().len() as u128)
                    .stats("update_latency_ms", stats(latency))
                    .stats("update_work_ms", stats(work))
                    .frames(update_begin.get(), end)
                    .finish();
                phase.set(Phase::Done);
                return false;
            }
            _ => {}
        }
        true
    });
    column((text(format!("{name}: {rows} rows")), show(shown, list)))
        .style(style().padding(8).gap(4).grow(1).min_height(0))
        .into_view()
}
