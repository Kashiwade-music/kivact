//! Role: Performance targets of the reactive core.
//! Run in release:
//! `cargo test -p kivact-public-api-tests --release --test e2e -- perf:: --ignored --nocapture`

use kivact::view::View;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

use kivact::control::canvas;
use kivact::draw::{Draw, Rect, Size};
use kivact::prelude::*;
use kivact::primitive::NodeRefExt as _;
use kivact::style::Color;
use kivact::test::render;

type Handles = Rc<Cell<Option<(Signal<u32>, Value<u32>)>>>;

const N: u32 = 2_000_000;

#[test]
#[ignore = "release-mode measurement"]
fn signal_get_tracked_and_set_with_one_subscriber() {
    let tracked_ns = Rc::new(Cell::new(0.0f64));
    let t2 = tracked_ns.clone();
    let slot: Handles = Rc::new(Cell::new(None));
    let s2 = slot.clone();
    let screen = render(move || {
        let s = signal(1u32);
        let probe = memo(move || {
            let start = Instant::now();
            let mut acc = 0u32;
            for _ in 0..N {
                acc = acc.wrapping_add(s.get());
            }
            t2.set(start.elapsed().as_nanos() as f64 / f64::from(N));
            acc
        });
        let sub = memo(move || s.get().wrapping_mul(3));
        s2.set(Some((s, sub)));
        text(move || probe.get().to_string())
    });
    let (s, sub) = slot.get().expect("mounted");
    let get_tracked = tracked_ns.get();

    let start = Instant::now();
    let mut acc = 0u32;
    for _ in 0..N {
        acc = acc.wrapping_add(s.get_untracked());
    }
    let get_untracked = start.elapsed().as_nanos() as f64 / f64::from(N);

    let _ = sub.get_untracked();
    let start = Instant::now();
    for i in 0..N {
        s.set(i + 10);
    }
    let set_amortized = start.elapsed().as_nanos() as f64 / f64::from(N);

    let start = Instant::now();
    for i in 0..N {
        s.set(i + 10 + N);
        acc = acc.wrapping_add(sub.get_untracked());
    }
    let set_and_pull = start.elapsed().as_nanos() as f64 / f64::from(N);
    screen.run_until_idle();

    println!(
        "PERF get_tracked_ns={get_tracked:.1} get_untracked_ns={get_untracked:.1} set_1sub_amortized_ns={set_amortized:.1} set_1sub_plus_pull_ns={set_and_pull:.1} (acc {acc})"
    );
    if !cfg!(debug_assertions) {
        assert!(
            get_tracked <= 30.0,
            "Signal::get tracked {get_tracked:.1} ns > 30 ns"
        );
        assert!(
            set_and_pull <= 200.0,
            "Signal::set with 1 subscriber {set_and_pull:.1} ns > 200 ns"
        );
    }
}

const BOXES: usize = 10_000;

#[test]
#[ignore = "release-mode canvas measurement"]
fn canvas_5000_one_pixel_rectangles() {
    let slot = Rc::new(Cell::new(None));
    let setup = slot.clone();
    let screen = render(move || {
        let tick = signal(0u32);
        setup.set(Some(tick));
        canvas(move |d: &mut Draw, _: Size| {
            let shift = tick.get() % 2;
            for i in 0..5000 {
                d.fill_rect(
                    Rect::new((i % 100 + shift) as f32, (i / 100) as f32, 1.0, 1.0),
                    Color::WHITE,
                );
            }
        })
        .style(style().width(128).height(64))
    });
    let tick = slot.get().expect("mounted");
    screen.run_until_idle();
    let start = Instant::now();
    for i in 1..=60 {
        tick.set(i);
        screen.run_until_idle();
        let _ = screen.screenshot();
    }
    println!(
        "PERF canvas_5000_rectangles redraw_ms={:.3}",
        start.elapsed().as_secs_f64() * 1000.0 / 60.0
    );
    assert!(screen.errors().is_empty());
}

/// One mount: wall ns, ops, live graph nodes, constants, allocated arena slots.
fn mount_boxes(f: fn() -> View) -> (u128, u64, usize, usize, usize) {
    let start = Instant::now();
    let screen = render(f);
    let ns = start.elapsed().as_nanos();
    let s = screen.stats();
    (ns, s.ops, s.live_reactive, s.live_constants, s.arena_slots)
}

fn plain_boxes() -> View {
    column(
        (0..BOXES)
            .map(|_| node().name("label").child(text("label")).into_view())
            .collect::<Vec<_>>(),
    )
    .into_view()
}

fn into_value_boxes() -> View {
    column(
        (0..BOXES)
            .map(|_| {
                let c = "label".into_value();
                node().name(c).child(text(c)).into_view()
            })
            .collect::<Vec<_>>(),
    )
    .into_view()
}

/// A label passed through `into_value` (one
/// constant arena entry per box, read by two props) mounts with the same ops
/// and no extra effect compared with plain constants. Runs alternate; the
/// median of each is printed.
#[test]
#[ignore = "release-mode measurement"]
fn mount_10k_labelled_boxes_constant_vs_into_value() {
    const RUNS: usize = 9;
    let (mut plain, mut into) = (Vec::new(), Vec::new());
    let (mut plain_ops, mut into_ops) = (0, 0);
    let (mut plain_nodes, mut into_nodes) = (0, 0);
    let (mut plain_consts, mut into_consts) = (0, 0);
    let (mut plain_slots, mut into_slots) = (0, 0);
    // Warm-up of both paths.
    let _ = (mount_boxes(plain_boxes), mount_boxes(into_value_boxes));
    for _ in 0..RUNS {
        let (ns, ops, nodes, consts, slots) = mount_boxes(plain_boxes);
        plain.push(ns);
        (plain_ops, plain_nodes, plain_consts) = (ops, nodes, consts);
        plain_slots = slots;
        let (ns, ops, nodes, consts, slots) = mount_boxes(into_value_boxes);
        into.push(ns);
        (into_ops, into_nodes, into_consts) = (ops, nodes, consts);
        into_slots = slots;
    }
    plain.sort_unstable();
    into.sort_unstable();
    let (p, i) = (plain[RUNS / 2], into[RUNS / 2]);
    println!(
        "PERF mount_10k_boxes plain_ms={:.2} into_value_ms={:.2} ratio={:.3} ops={plain_ops}/{into_ops} live_reactive={plain_nodes}/{into_nodes} live_constants={plain_consts}/{into_consts} arena_slots={plain_slots}/{into_slots}",
        p as f64 / 1e6,
        i as f64 / 1e6,
        i as f64 / p as f64
    );
    assert_eq!(plain_ops, into_ops, "same ops");
    assert_eq!(into_nodes, plain_nodes, "constants are not graph nodes");
    assert!(plain_nodes < 100, "literal labels have bounded graph size");
    assert!(into_nodes < 100, "constant Values have bounded graph size");
    assert_eq!(
        into_consts,
        plain_consts + BOXES,
        "each Value owns one constant entry"
    );
    assert_eq!(
        into_slots,
        plain_slots + BOXES,
        "constant entries still count as arena allocations"
    );
}

/// `lines` lines of about 30 bytes each.
fn long_document(lines: usize) -> String {
    (0..lines)
        .map(|i| format!("line {i} of a long document\n"))
        .collect()
}

/// One character typed into the middle of a 100 000-line `text_area`: the
/// median time the core and the host spend from the key to the idle screen,
/// over 200 keystrokes after a warm-up. Recorded, not gated (the work order
/// keeps about 10 ms per character).
#[test]
#[ignore = "release-mode measurement"]
fn text_area_100k_lines_one_char() {
    const LINES: usize = 100_000;
    const WARM_UP: usize = 20;
    const KEYS: usize = 200;
    let text = long_document(LINES);
    let middle = text.len() / 2;
    let slot = Rc::new(Cell::new(None));
    let setup = slot.clone();
    let screen = render(move || {
        let value = signal(text.clone());
        let reference = node_ref();
        setup.set(Some((value, reference)));
        kivact::ui::text_area("Doc", value)
            .node_ref(reference)
            .style(style().size(400, 300))
    });
    let (value, reference) = slot.get().expect("mounted");
    let field = screen.get_by_role(Role::TextBox, "Doc");
    field.focus();
    // The caret goes to the middle of the document.
    let at = value.with_untracked(|t| t[..middle].rfind('\n').map_or(0, |i| i + 1));
    reference.set_edit_session(kivact::primitive::TextSession::caret_at(at));
    screen.run_until_idle();
    let mut samples = Vec::with_capacity(KEYS);
    for i in 0..WARM_UP + KEYS {
        let start = Instant::now();
        field.type_text("x");
        screen.run_until_idle();
        if i >= WARM_UP {
            samples.push(start.elapsed());
        }
    }
    samples.sort_unstable();
    let median = samples[KEYS / 2].as_secs_f64() * 1000.0;
    let worst = samples[KEYS - 1].as_secs_f64() * 1000.0;
    println!(
        "PERF text_area_100k_lines_one_char median_ms={median:.3} worst_ms={worst:.3} lines={LINES} keys={KEYS}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    assert_eq!(
        value.with_untracked(|t| t.matches('x').count()),
        WARM_UP + KEYS
    );
}

/// Retain all rows: this measures accessibility generation, not virtualization.
#[test]
#[ignore = "release-mode accessibility work measurement"]
fn accessibility_static_list_typing() {
    let mut measured = Vec::new();
    for rows in [1_000, 10_000] {
        let slot = Rc::new(Cell::new(None));
        let setup = slot.clone();
        let screen = render(move || {
            let value = signal(String::new());
            setup.set(Some(value));
            column((
                kivact::ui::text_input("Probe", value).style(style().size(300, 40)),
                column(
                    (0..rows)
                        .map(|i| text(format!("row {i}")).into_view())
                        .collect::<Vec<_>>(),
                )
                .style(style().width(300).height(300)),
            ))
        });
        let value = slot.get().expect("mounted");
        let field = screen.get_by_role(Role::TextBox, "Probe");
        field.focus();
        field.type_text("warm");
        screen.run_until_idle();
        for ch in ["a", "b", "c", "d", "e"] {
            let old = value.get_untracked();
            let before = screen.stats();
            let start = Instant::now();
            field.type_text(ch);
            screen.run_until_idle();
            let ms = start.elapsed().as_secs_f64() * 1000.0;
            let after = screen.stats();
            let constructed = after.a11y_constructed - before.a11y_constructed;
            let published = after.a11y_published - before.a11y_published;
            let expected = format!("{old}{ch}");
            let current = value.get_untracked();
            let accepted = field.text_runs().unwrap_or_default().concat();
            println!(
                "PERF a11y_static rows={rows} char={ch} constructed={constructed} published={published} ms={ms:.3} value_ok={} accepted_ok={}",
                current == expected,
                accepted == expected
            );
            measured.push((
                rows,
                constructed,
                published,
                current == expected,
                accepted == expected,
            ));
        }
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
    for (rows, constructed, published, value_ok, accepted_ok) in measured {
        assert!(
            value_ok && accepted_ok,
            "rows={rows}: typed value must reach accepted text"
        );
        assert!(
            (1..=64).contains(&constructed) && (1..=64).contains(&published),
            "rows={rows}: constructed={constructed}, published={published}, expected 1..=64"
        );
    }
}
