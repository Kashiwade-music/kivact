//! Role: Tests for text layout and wrapping behavior.
//!
//! The layout check on real windows with real shaping: a text that wraps in
//! a narrow button (a Japanese one, in Noto Sans JP), overflows its parent,
//! sits above the middle of its row or is cut by a clip is found with where
//! it was created; an ellipsis, a scrolling axis and an ignored test id are
//! not violations; a text of white space alone is one line high in any
//! width; `Element::check_layout` looks at a subtree;
//! `assert_layout` fails listing every violation; and in debug builds each
//! defect is warned about once on the `kivact::layout` tracing target.

use std::sync::Arc;
#[cfg(debug_assertions)]
use std::sync::Mutex;

#[cfg(debug_assertions)]
use crate::support::mount_with;
use kivact::control::{rich_text, span};
use kivact::prelude::*;
use kivact::style::{Align, Justify};
use kivact::style::{Overflow, Position, TextOverflow};
use kivact::test::{
    LayoutCheck, LayoutRule, LayoutViolation, RenderOptions, TestFonts, render, render_with,
};

static NOTO: &[u8] = include_bytes!(
    "../../../../../examples/kivact-ui-pop/assets/fonts/notosansjp/NotoSansJP[wght].ttf"
);

const GRAY: u32 = 0xDDDDDD;

fn japanese() -> RenderOptions {
    RenderOptions {
        fonts: TestFonts::Only(vec![Arc::from(NOTO)]),
        ..RenderOptions::from_env()
    }
}

/// A fixed-width button whose label does not fit (its label is located at
/// the call).
#[track_caller]
fn narrow_button(label: &'static str) -> Node {
    row((text(label).test_id("label"),))
        .role(Role::Button)
        .name(label)
        .style(
            style()
                .width(52)
                .height(30)
                .padding_x(10)
                .align_items(Align::Center)
                .background(rgb(GRAY)),
        )
}

fn rules(found: &[LayoutViolation]) -> Vec<LayoutRule> {
    found.iter().map(|v| v.rule).collect()
}

fn overflows_parent(rule: &LayoutRule) -> bool {
    matches!(rule, LayoutRule::OverflowVisualParent)
}

#[test]
fn a_narrow_japanese_button_is_wrapped_and_overflows() {
    let line = line!() + 1;
    let screen = render_with(japanese(), || narrow_button("保存して閉じる"));
    let found = screen.check_layout();
    let wrapped = found
        .iter()
        .find(|v| matches!(v.rule, LayoutRule::Wrapped { lines } if lines >= 2))
        .unwrap_or_else(|| panic!("no Wrapped in {found:#?}"));
    assert_eq!(wrapped.text, "保存して閉じる");
    assert_eq!(wrapped.role, Some(Role::Text));
    assert_eq!(wrapped.test_id.as_deref(), Some("label"));
    assert!(wrapped.amount > 0.0);
    assert!(
        wrapped.created_at.file().ends_with("layout_check.rs"),
        "{}",
        wrapped.created_at
    );
    assert_eq!(wrapped.created_at.line(), line);
    let overflow = found
        .iter()
        .find(|v| overflows_parent(&v.rule))
        .unwrap_or_else(|| panic!("no Overflow in {found:#?}"));
    assert!(overflow.amount > 1.0, "{overflow}");
    // The 30 px button is the limit the lines are higher than.
    assert_eq!(overflow.limit.height, 30.0);
    assert!(overflow.text_rect.height > 30.0);
    let line = wrapped.to_string();
    assert!(line.contains("layout_check.rs"), "{line}");
    assert!(line.contains("保存して閉じる"), "{line}");
    assert!(!line.contains('\n'), "{line}");
    assert!(screen.errors().is_empty());
}

#[test]
fn a_nowrap_text_wider_than_its_row_overflows_it() {
    let screen = render(|| {
        column((
            row((text("Quarterly revenue summary")
                .style(style().text_wrap(false))
                .test_id("long"),))
            .style(
                style()
                    .width(80)
                    .align_items(Align::Start)
                    .background(rgb(GRAY)),
            ),
            // A control that fits.
            row((text("Short").test_id("short"),)).style(
                style()
                    .width(80)
                    .align_items(Align::Start)
                    .background(rgb(GRAY)),
            ),
        ))
        .style(style().align_items(Align::Start))
    });
    let found = screen.check_layout();
    assert_eq!(found.len(), 1, "{found:#?}");
    let v = &found[0];
    assert_eq!(v.test_id.as_deref(), Some("long"));
    assert!(overflows_parent(&v.rule), "{v}");
    // The row is 80 wide, the text more than twice that.
    assert!(v.amount > 60.0, "{v}");
    assert_eq!(v.limit.width, 80.0);
}

#[test]
fn a_text_above_the_middle_of_its_row_is_off_center() {
    let screen = render(|| {
        column((
            row((text("Title").style(style().height(30)).test_id("high"),)).style(
                style()
                    .height(44)
                    .align_items(Align::Center)
                    .background(rgb(GRAY)),
            ),
            // Out of the flow, at its insets: the row does not center it.
            row((text("Title").style(style().position(Position::Absolute).top(2)),)).style(
                style()
                    .height(44)
                    .align_items(Align::Center)
                    .background(rgb(GRAY)),
            ),
            row((text("Title").test_id("middle"),)).style(
                style()
                    .height(44)
                    .align_items(Align::Center)
                    .background(rgb(GRAY)),
            ),
        ))
    });
    let found = screen.check_layout();
    assert_eq!(found.len(), 1, "{found:#?}");
    let v = &found[0];
    assert_eq!(v.rule, LayoutRule::OffCenter);
    assert_eq!(v.test_id.as_deref(), Some("high"));
    // The line sits at the top of its 30 px box: about (30 - 18) / 2 high.
    assert!((v.amount - 6.0).abs() < 2.0, "{v}");
    // Moving the text to the middle of its box mends it.
    assert!(screen.get_by_test_id("middle").check_layout().is_empty());
}

#[test]
fn a_clip_cutting_a_text_with_no_ellipsis_is_one_violation() {
    let screen = render(|| {
        row((text("A long line of text that cannot fit its box")
            .style(style().text_wrap(false))
            .test_id("cut"),))
        .style(
            style()
                .width(80)
                .align_items(Align::Start)
                .overflow(Overflow::Hidden),
        )
    });
    let found = screen.check_layout();
    assert_eq!(
        rules(&found),
        [LayoutRule::ClippedWithoutEllipsis],
        "{found:#?}"
    );
    assert!(found[0].amount > 100.0, "{}", found[0]);
    assert_eq!(found[0].limit.width, 80.0);
}

#[test]
fn an_ellipsis_a_scrolling_axis_and_a_text_that_fits_are_not_violations() {
    let screen = render(|| {
        column((
            text("A long line of text that cannot fit its box")
                .style(
                    style()
                        .width(160)
                        .text_wrap(false)
                        .text_overflow(TextOverflow::Ellipsis)
                        .overflow(Overflow::Hidden),
                )
                .test_id("ellipsis"),
            // Cut by the row's clip, and marked by the ellipsis.
            row((text("A long line of text that cannot fit its box")
                .style(
                    style()
                        .grow(1)
                        .text_wrap(false)
                        .text_overflow(TextOverflow::Ellipsis),
                )
                .test_id("shrunk"),))
            .style(style().width(120).overflow(Overflow::Hidden)),
            scroll(
                text("A long line of text that cannot fit its box")
                    .style(style().text_wrap(false))
                    .test_id("sideways"),
            )
            .orientation(Orientation::Horizontal)
            .style(style().width(100).background(rgb(GRAY))),
            scroll(column(
                (0..12)
                    .map(|i| text(format!("Row number {i}")))
                    .collect::<Vec<_>>(),
            ))
            .style(style().width(100).height(60).background(rgb(GRAY))),
            text("Short").test_id("short"),
        ))
        .style(style().align_items(Align::Start))
    });
    assert_eq!(screen.check_layout(), []);
    screen.assert_layout();
}

#[test]
fn a_text_of_white_space_alone_is_one_line_high_in_any_width() {
    let screen = render(|| {
        column((
            text("x").test_id("letter"),
            text(" ").test_id("space"),
            text("    ")
                .style(style().text_wrap(false))
                .test_id("spaces"),
            row(text("  ").style(style().width(0)).test_id("zero")),
            rich_text(|| vec![span("   ")]).test_id("rich"),
        ))
        .style(style().align_items(Align::Start))
    });
    let height = |id: &str| screen.get_by_test_id(id).rect().expect("laid out").height;
    let line = height("letter");
    for id in ["space", "spaces", "zero", "rich"] {
        assert_eq!(height(id), line, "{id}");
    }
    let found = screen.check_layout();
    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn single_line_cuts_a_long_label_with_an_ellipsis_in_a_narrow_box() {
    let long = "A long line of text that cannot fit its box";
    let screen = render(move || {
        column((
            // In a flex row the label shrinks to the room it has.
            row((text(long).style(style().single_line()).test_id("shrunk"),))
                .style(style().width(100).min_width(0)),
            text(long)
                .style(style().width(100).single_line())
                .test_id("fixed"),
            // The same labels without it are defects.
            text(long)
                .style(style().width(100).text_wrap(false))
                .test_id("plain"),
        ))
        .style(style().align_items(Align::Start))
    });
    let found = screen.check_layout();
    assert!(
        found.iter().all(|v| v.test_id.as_deref() == Some("plain")),
        "{found:#?}"
    );
    assert!(!found.is_empty(), "the plain label should be a violation");
    let check = LayoutCheck::default().ignore_test_id("plain");
    assert_eq!(screen.check_layout_with(&check), []);
}

#[test]
fn ignored_test_ids_and_the_tolerances_are_options() {
    let screen = render_with(japanese(), || {
        column((
            narrow_button("保存して閉じる").test_id("ignored"),
            row((text("Title").style(style().height(30)),))
                .style(
                    style()
                        .height(44)
                        .align_items(Align::Center)
                        .background(rgb(GRAY)),
                )
                .test_id("bar"),
        ))
    });
    let all = screen.check_layout();
    assert!(all.len() >= 3, "{all:#?}");
    let mut check = LayoutCheck::default().ignore_test_id("ignored");
    let rest = screen.check_layout_with(&check);
    assert_eq!(rules(&rest), [LayoutRule::OffCenter], "{rest:#?}");
    check.center_tolerance = Some(20.0);
    assert_eq!(screen.check_layout_with(&check), []);
    // A button is no longer a single line, and the lines may be higher.
    let mut lenient = LayoutCheck::default();
    lenient.overflow_tolerance = 1000.0;
    lenient.single_line_roles.clear();
    lenient.center_tolerance = Some(20.0);
    assert_eq!(screen.check_layout_with(&lenient), []);
}

#[test]
fn an_element_checks_only_its_subtree() {
    let screen = render_with(japanese(), || {
        column((
            narrow_button("保存して閉じる").test_id("first"),
            narrow_button("削除して戻る").test_id("second"),
            text("fine"),
        ))
    });
    let all = screen.check_layout();
    let first = screen.get_by_test_id("first").check_layout();
    let second = screen.get_by_test_id("second").check_layout();
    assert!(!first.is_empty() && !second.is_empty());
    assert_eq!(first.len() + second.len(), all.len(), "{all:#?}");
    assert!(
        first.iter().all(|v| v.text == "保存して閉じる"),
        "{first:#?}"
    );
    assert!(
        second.iter().all(|v| v.text == "削除して戻る"),
        "{second:#?}"
    );
}

#[test]
fn assert_layout_fails_listing_every_violation() {
    let screen = render_with(japanese(), || {
        column((
            narrow_button("保存して閉じる"),
            narrow_button("削除して戻る"),
        ))
    });
    let expected = screen.check_layout().len();
    assert!(expected >= 4, "{expected}");
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| screen.assert_layout()))
        .expect_err("violations fail the test");
    let message = caught
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| caught.downcast_ref::<&str>().map(|s| s.to_string()))
        .expect("a message");
    assert!(
        message.starts_with(&format!("{expected} layout violation(s):")),
        "{message}"
    );
    assert_eq!(
        message
            .lines()
            .filter(|l| l.contains("layout_check.rs"))
            .count(),
        expected,
        "{message}"
    );
    assert!(message.contains("保存して閉じる") && message.contains("削除して戻る"));
    assert!(message.contains("failure artifacts"), "{message}");
}

#[test]
fn a_healthy_screen_passes() {
    let screen = render(|| {
        column((
            row((text("Save"),)).role(Role::Button).name("Save").style(
                style()
                    .height(32)
                    .padding_x(12)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .background(rgb(GRAY)),
            ),
            row((kivact::control::node().name("Name").part(kivact::interaction::use_text_edit(kivact::signal(String::new())).control()),)).style(style().align_items(Align::Center).background(rgb(GRAY))),
            text("A paragraph of text that wraps over several lines because it is far wider than its column.")
                .style(style().width(160)),
        ))
        .style(style().align_items(Align::Start).gap(8))
    });
    screen.assert_layout();
}

/// Warnings of the `kivact::layout` target, from a subscriber of this
/// thread (the screen and its host run on it).
#[cfg(debug_assertions)]
#[derive(Clone, Default)]
struct Warnings(Arc<Mutex<Vec<String>>>);

#[cfg(debug_assertions)]
impl tracing::Subscriber for Warnings {
    fn enabled(&self, m: &tracing::Metadata<'_>) -> bool {
        m.target() == "kivact::layout"
    }
    fn new_span(&self, _: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        tracing::span::Id::from_u64(1)
    }
    fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}
    fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}
    fn event(&self, e: &tracing::Event<'_>) {
        struct Message(String);
        impl tracing::field::Visit for Message {
            fn record_debug(&mut self, f: &tracing::field::Field, v: &dyn std::fmt::Debug) {
                if f.name() == "message" {
                    self.0 = format!("{v:?}");
                }
            }
        }
        let mut message = Message(String::new());
        e.record(&mut message);
        if let Ok(mut all) = self.0.lock() {
            all.push(message.0);
        }
    }
    fn enter(&self, _: &tracing::span::Id) {}
    fn exit(&self, _: &tracing::span::Id) {}
}

#[cfg(debug_assertions)]
#[test]
fn a_defect_is_warned_about_once_until_the_text_or_its_box_changes() {
    let warnings = Warnings::default();
    tracing::subscriber::with_default(warnings.clone(), || {
        let (screen, label) = mount_with(japanese(), || {
            let label = signal(String::from("保存して閉じる"));
            (
                label,
                column((
                    row((text(move || label.get()),))
                        .role(Role::Button)
                        .name("Save")
                        .style(
                            style()
                                .width(52)
                                .height(30)
                                .padding_x(10)
                                .align_items(Align::Center)
                                .background(rgb(GRAY)),
                        ),
                    text("fine"),
                )),
            )
        });
        let count = || warnings.0.lock().expect("lock").len();
        let expected = screen.check_layout().len();
        assert!(expected >= 2, "{expected}");
        // One warning for each rule of the one defective text.
        assert_eq!(count(), expected, "{:?}", warnings.0.lock());
        {
            let all = warnings.0.lock().expect("lock");
            assert!(all.iter().all(|m| m.contains("保存して閉じる")), "{all:?}");
            assert!(all.iter().any(|m| m.contains("Wrapped")), "{all:?}");
            assert!(all.iter().all(|m| m.contains("layout_check.rs")), "{all:?}");
        }
        // Settling again, hovering and a frame report nothing more.
        screen.get_by_text("fine").hover();
        screen.advance(std::time::Duration::from_millis(100));
        let _ = screen.screenshot();
        assert_eq!(count(), expected);
        // New content is checked again.
        label.set("削除して戻る".into());
        screen.run_until_idle();
        assert!(count() > expected, "{:?}", warnings.0.lock());
        assert!(
            warnings
                .0
                .lock()
                .expect("lock")
                .last()
                .is_some_and(|m| m.contains("削除して戻る"))
        );
        // A lint is no error of the App.
        assert!(screen.errors().is_empty());
    });
}

#[cfg(debug_assertions)]
#[test]
fn a_screen_without_defects_warns_about_nothing() {
    let warnings = Warnings::default();
    tracing::subscriber::with_default(warnings.clone(), || {
        let screen = render(|| {
            column((
                text("Title"),
                row((text("Save"),)).role(Role::Button).name("Save"),
            ))
        });
        screen.run_until_idle();
        assert_eq!(*warnings.0.lock().expect("lock"), Vec::<String>::new());
    });
}
