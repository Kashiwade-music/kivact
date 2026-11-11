//! Role: Update-latency probe on the test driver: input to drawn frame, by list size.
//!
//! Run with `KIVACT_TEST_MIRROR=off`. Clicks go to window coordinates
//! (`click_at`) so no query cost is measured. Each `L` line is one
//! scenario in milliseconds (input, turn, layout, paint, raster on the CPU,
//! accessibility publication of the test driver).

#![allow(
    clippy::print_stderr,
    reason = "the probe reports its measurements on the console"
)]

use kivact::prelude::*;
use kivact::test::Screen;
use kivact_bench::report;
use std::time::Instant;

#[derive(Clone, PartialEq)]
struct Row {
    id: u32,
    label: String,
}

fn rows(n: u32) -> Vec<Row> {
    (0..n)
        .map(|id| Row {
            id,
            label: format!("Row {id}"),
        })
        .collect()
}

/// Milliseconds of `f`, `times` times.
fn time(times: usize, mut f: impl FnMut(usize)) -> Vec<f64> {
    (0..times)
        .map(|i| {
            let t = Instant::now();
            f(i);
            t.elapsed().as_secs_f64() * 1e3
        })
        .collect()
}

/// The window center of the button named `name`.
fn center(screen: &Screen, name: &str) -> (f64, f64) {
    let b = screen
        .get_by_role(Role::Button, name)
        .accessibility_bounds()
        .expect("invariant: the probe's buttons have bounds");
    (b.x + b.width / 2.0, b.y + b.height / 2.0)
}

fn list(n: u32) -> impl IntoView {
    let list = signal(rows(n));
    let other = signal(0);
    column((
        text(move || format!("Other {}", other.get())),
        button("Other").on_press(move || other.update(|n| *n += 1)),
        button("Bump").on_press(move || list.update(|l| l[0].label.push('!'))),
        scroll(column(for_each(
            list,
            |r| r.id,
            move |r| row((text(move || r.get().label), button("x"))).style(style().gap(4)),
        ))),
    ))
}

fn counter() -> impl IntoView {
    let count = signal(0);
    column((
        text(move || format!("Count: {}", count.get())),
        button("Increment").on_press(move || count.update(|n| *n += 1)),
    ))
}

fn probe_list(n: u32) {
    let t = Instant::now();
    let screen = kivact::test::render(move || list(n));
    report(
        &format!("mount_n{n}"),
        vec![t.elapsed().as_secs_f64() * 1e3],
    );
    let (ox, oy) = center(&screen, "Other");
    report(
        &format!("unrelated_update_n{n}"),
        time(50, |_| screen.click_at(ox, oy)),
    );
    let (bx, by) = center(&screen, "Bump");
    report(
        &format!("row_update_n{n}"),
        time(50, |_| screen.click_at(bx, by)),
    );
    report(
        &format!("pointer_move_n{n}"),
        time(120, |i| {
            screen.mouse_move(40.0 + (i % 7) as f64, 80.0 + (i as f64) * 3.0)
        }),
    );
    report(
        &format!("wheel_n{n}"),
        time(40, |_| screen.scroll(0.0, 40.0)),
    );
    eprintln!("S stats_n{n} {:?}", screen.stats());
}

fn main() {
    let t = Instant::now();
    let screen = kivact::test::render(counter);
    report("mount_counter", vec![t.elapsed().as_secs_f64() * 1e3]);
    let (x, y) = center(&screen, "Increment");
    report("counter_click", time(200, |_| screen.click_at(x, y)));
    drop(screen);
    for n in [100u32, 1000, 3000] {
        probe_list(n);
    }
}
