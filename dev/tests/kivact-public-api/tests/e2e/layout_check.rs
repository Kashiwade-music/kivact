//! Role: Tests for text layout and wrapping behavior.
//!
//! The layout check on real windows with real shaping: a text that wraps in
//! a narrow button (a Japanese one, in Noto Sans JP), overflows its parent,
//! sits above the middle of its row or is cut by a clip is found with where
//! it was created; an ellipsis, a scrolling axis and an ignored test id are
//! not violations; a text of white space alone is one line high in any
//! width; `Locator::check_layout` looks at a subtree; and
//! `assert_layout` fails listing every violation.

use kivact::prelude::*;
use kivact::style::{Align, Justify};
use kivact::style::{Overflow, TextOverflow};
use kivact::test::{LayoutLimit, LayoutRule, RenderOptions, TestFonts, render, render_with};

static NOTO: &[u8] =
    include_bytes!("../../../../../examples/shared-assets/fonts/notosansjp/NotoSansJP[wght].ttf");

const GRAY: u32 = 0xdddddd;

fn japanese() -> RenderOptions {
    RenderOptions::from_env().fonts(TestFonts::Only(vec![kivact::view::Asset::from(NOTO)]))
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

fn overflows_parent(rule: &LayoutRule) -> bool {
    matches!(
        rule,
        LayoutRule::Overflow {
            into: LayoutLimit::VisualParent
        }
    )
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
    assert!(overflows_parent(&v.rule()), "{v}");
    // The row is 80 wide, the text more than twice that.
    assert!(v.amount() > 60.0, "{v}");
    assert_eq!(v.limit().width, 80.0);
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
            .axes(ScrollAxes::Horizontal)
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
fn check_layout_lists_every_violation() {
    let screen = render_with(japanese(), || {
        column((
            narrow_button("保存して閉じる"),
            narrow_button("削除して戻る"),
        ))
    });
    let violations: Vec<String> = screen
        .check_layout()
        .iter()
        .map(|v| v.to_string())
        .collect();
    assert!(violations.len() >= 4, "{violations:?}");
    assert!(
        violations.iter().all(|v| v.contains("layout_check.rs")),
        "{violations:?}"
    );
    let all = violations.join(
        "
",
    );
    assert!(
        all.contains("保存して閉じる") && all.contains("削除して戻る"),
        "{all}"
    );
}

#[test]
#[should_panic(expected = "layout violation(s):")]
fn assert_layout_panics_on_violations() {
    let screen = render_with(japanese(), || {
        column((
            narrow_button("保存して閉じる"),
            narrow_button("削除して戻る"),
        ))
    });
    screen.assert_layout();
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
            row((kivact::view::node().name("Name").part(kivact::interaction::use_text_edit(kivact::state::signal(String::new())).control()),)).style(style().align_items(Align::Center).background(rgb(GRAY))),
            text("A paragraph of text that wraps over several lines because it is far wider than its column.")
                .style(style().width(160)),
        ))
        .style(style().align_items(Align::Start).gap(8))
    });
    screen.assert_layout();
}
