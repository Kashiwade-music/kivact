//! Role: Tests for the turn rules: what one turn settles, and its limits.
//! A focus result or a layout report is delivered in one turn that also
//! commits what its handlers and effects write (no extra turn or wake);
//! loops that keep a turn or the host busy are cut with a `Cycle`
//! diagnostic, and the App finishes the work afterwards.

use std::time::Duration;

use crate::support::mount;
use kivact::control::Node;
use kivact::diagnostics::ErrorKind;
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::Screen;

fn cycles(screen: &Screen, needle: &str) -> usize {
    screen
        .errors()
        .iter()
        .filter(|e| matches!(e.kind(), ErrorKind::Cycle { .. }) && e.message().contains(needle))
        .count()
}

fn button(label: &'static str, press: impl Fn() + 'static) -> Node {
    node()
        .role(Role::Button)
        .focusable(true)
        .name(label)
        .child(text(label))
        .on_press(press)
}

#[test]
fn a_focus_result_settles_its_effects_in_the_turn_that_delivers_it() {
    let (screen, ()) = mount(|| {
        let target = node_ref();
        let focused = target.focused();
        let seen = signal(0);
        let label = signal(String::from("waiting"));
        // A user effect of the focus source, and one of what it writes.
        effect(move || {
            if focused.get() {
                seen.update(|n| *n += 1);
            }
        });
        effect(move || {
            let n = seen.get();
            if n > 0 {
                label.set(format!("seen {n}"));
            }
        });
        (
            (),
            column((
                button("go", move || target.focus()),
                button("target", || {}).node_ref(target),
                text(move || label.get()),
            )),
        )
    });
    let before = screen.stats().turns;
    screen.get_by_role(Role::Button, "go").click();
    screen.get_by_text("seen 1");
    assert!(screen.get_by_role(Role::Button, "target").is_focused());
    // The click's turn requests the focus; the next turn delivers the
    // focus result and commits both effects' writes.
    assert_eq!(
        screen.stats().turns - before,
        2,
        "the focus result and its effects cost one turn"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_layout_report_settles_its_effects_in_the_turn_that_delivers_it() {
    let (screen, width) = mount(|| {
        let width = signal(40.0f32);
        let boxed = node_ref();
        let rect = boxed.rect();
        let label = signal(String::new());
        effect(move || {
            if let Some(r) = rect.get() {
                label.set(format!("width {}", r.width));
            }
        });
        (
            width,
            column((
                node()
                    .node_ref(boxed)
                    .style(move || style().width(width.get()).height(10)),
                text(move || label.get()),
            ))
            .style(style().align_items(Align::Start)),
        )
    });
    screen.get_by_text("width 40");
    let before = screen.stats().turns;
    width.set(64.0);
    screen.run_until_idle();
    screen.get_by_text("width 64");
    // The write's turn commits the width; the next turn delivers the
    // layout report and commits the label.
    assert_eq!(screen.stats().turns - before, 2);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_layout_feedback_loop_is_cut_and_reported_then_finishes() {
    const END: f32 = 150.0;
    let (screen, width) = mount(|| {
        let width = signal(10.0f32);
        let boxed = node_ref();
        let rect = boxed.rect();
        // Every layout report widens the box by one: one turn per report.
        effect(move || {
            if rect.get().is_some() && width.get_untracked() < END {
                width.update(|w| *w += 1.0);
            }
        });
        (
            width,
            column((node()
                .node_ref(boxed)
                .style(move || style().width(width.get()).height(10)),))
            .style(style().align_items(Align::Start)),
        )
    });
    screen.run_until_idle();
    assert_eq!(
        width.get_untracked(),
        END,
        "the loop finishes after the cut"
    );
    assert!(
        cycles(&screen, "turns in a row") >= 1,
        "the host's cut is reported: {:?}",
        screen.errors()
    );
}

#[test]
fn tasks_that_keep_one_turn_busy_are_cut_and_reported_then_finish() {
    const END: u32 = 1500;
    let (screen, n) = mount(|| {
        let n = signal(0u32);
        // Each fetch writes the source of the next: one task per round of
        // the turn's settling, far beyond its limit.
        let _chain = resource(
            move || n.get(),
            move |v: u32| async move {
                if v < END {
                    n.set(v + 1);
                }
                Ok(v)
            },
        );
        (n, text(move || format!("n {}", n.get())))
    });
    screen.run_until_idle();
    screen.advance(Duration::from_millis(10));
    assert_eq!(n.get_untracked(), END, "the chain finishes in later turns");
    screen.get_by_text(&format!("n {END}"));
    assert!(
        cycles(&screen, "tasks and timers") >= 1,
        "the cut is reported: {:?}",
        screen.errors()
    );
}
