//! Role: Tests for baseline alignment across text and primitives.
//!
//! `Align::Baseline` on real windows: a text, a text field and a primitive
//! that reports its baseline line up on their first baselines (a
//! primitive that reports none, on the bottom of its box), not on their
//! bottoms.

use std::cell::Cell;
use std::rc::Rc;

use kivact::control::canvas;
use kivact::draw::{Rect, Size};
use kivact::draw::{TextMetrics, TextStyle};
use kivact::prelude::*;
use kivact::primitive::{Constraints, Measurement, primitive};
use kivact::style::Align;
use kivact::test::Screen;

use crate::support::mount;

fn rect(screen: &Screen, id: &str) -> Rect {
    screen.get_by_test_id(id).rect().expect("laid out")
}

/// The metrics `Draw::measure_text` gives a one-line text of `size`
/// (what a text element of that font size is laid out with).
fn measure_in(d: &mut kivact::draw::Draw, size: f32) -> TextMetrics {
    d.measure_text(
        "Mg",
        &TextStyle {
            size,
            ..TextStyle::default()
        },
    )
}

fn close(a: f32, b: f32, what: &str) {
    assert!((a - b).abs() <= 1.0, "{what}: {a} vs {b}");
}

/// A 36 px value and a 16 px unit in a row aligned by baseline (the
/// game's display): the glyphs stand on one line, so the small text is
/// not at the bottom of the big one's box.
#[test]
fn texts_of_different_sizes_share_a_baseline_in_a_row() {
    let big: Rc<Cell<TextMetrics>> = Rc::default();
    let small: Rc<Cell<TextMetrics>> = Rc::default();
    let (b, s) = (big.clone(), small.clone());
    let (screen, ()) = mount(move || {
        let view = column((
            row((
                text("Mg").style(style().font_size(36)).test_id("big"),
                text("Mg").style(style().font_size(16)).test_id("small"),
            ))
            .style(style().align_items(Align::Baseline)),
            canvas(move |d, _| {
                b.set(measure_in(d, 36.0));
                s.set(measure_in(d, 16.0));
            })
            .style(style().size(10, 10)),
        ));
        ((), view)
    });
    let (big_box, small_box) = (rect(&screen, "big"), rect(&screen, "small"));
    let (big_line, small_line) = (big.get().baseline, small.get().baseline);
    assert!(big_line > small_line && small_line > 0.0);
    close(
        big_box.y + big_line,
        small_box.y + small_line,
        "the baselines of the two texts",
    );
    assert!(
        (big_box.y + big_box.height - (small_box.y + small_box.height)).abs() > 4.0,
        "the bottoms differ: {big_box:?} vs {small_box:?}"
    );
}

/// A text next to a 40 px tall text field: the text stands on the line the
/// field centers in its content box.
#[test]
fn a_text_and_a_text_field_share_a_baseline() {
    let line: Rc<Cell<TextMetrics>> = Rc::default();
    let out = line.clone();
    let (screen, ()) = mount(move || {
        let view = column((
            row((
                text("Name").style(style().font_size(16)).test_id("label"),
                kivact::control::node()
                    .name("Name")
                    .part(
                        kivact::interaction::use_text_edit(kivact::signal(String::new())).control(),
                    )
                    .style(style().font_size(16).height(40))
                    .test_id("field"),
            ))
            .style(style().align_items(Align::Baseline)),
            canvas(move |d, _| out.set(measure_in(d, 16.0))).style(style().size(10, 10)),
        ));
        ((), view)
    });
    let (label, field) = (rect(&screen, "label"), rect(&screen, "field"));
    let m = line.get();
    let field_baseline = field.y + (field.height - m.height) / 2.0 + m.baseline;
    close(
        label.y + m.baseline,
        field_baseline,
        "the label's baseline and the field's line",
    );
}

/// A primitive that reports a baseline stands it on the text's; one that
/// reports none stands its box bottom on it.
#[test]
fn a_primitive_stands_on_its_reported_baseline_or_its_bottom() {
    let line: Rc<Cell<TextMetrics>> = Rc::default();
    let out = line.clone();
    let (screen, ()) = mount(move || {
        let view = column((
            row((
                text("Mg").style(style().font_size(16)).test_id("label"),
                primitive()
                    .measure((), |_: &(), _: Constraints| {
                        Measurement::new(Size::new(20.0, 30.0)).baseline(22.0)
                    })
                    .test_id("with"),
                primitive()
                    .measure((), |_: &(), _: Constraints| Size::new(20.0, 30.0))
                    .test_id("without"),
            ))
            .style(style().align_items(Align::Baseline)),
            canvas(move |d, _| out.set(measure_in(d, 16.0))).style(style().size(10, 10)),
        ));
        ((), view)
    });
    let (label, with, without) = (
        rect(&screen, "label"),
        rect(&screen, "with"),
        rect(&screen, "without"),
    );
    let on_line = label.y + line.get().baseline;
    close(with.y + 22.0, on_line, "the reported baseline");
    close(without.y + without.height, on_line, "the bottom of the box");
}
