//! Role: Tests for the split in the kit's theme, property rows, windows and panels.
//!
//! The standard split, property rows and state buttons, windows an application opens
//! and closes, and panels with a header of their own.

use crate::support::{clean, near};
use kivact::draw::Rect;
use kivact::prelude::*;
use kivact::test::{Element, Screen};
use kivact_ui_creative as ui;

fn width(element: &Element<'_>) -> f64 {
    element.rect().expect("laid out").width
}

fn row_of(screen: &Screen, label: &str) -> Rect {
    screen
        .get_by_role(Role::Group, label)
        .rect()
        .expect("row is laid out")
}

#[test]
fn a_property_row_lays_out_label_leading_value_and_trailing_in_order() {
    let (screen, ()) = kivact::test::render_with_handle(crate::support::options(), || {
        (
            (),
            column((
                ui::property_row(
                    "Opacity",
                    kivact::ui::button("Value")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .test_id("value"),
                )
                .leading(|| {
                    kivact::ui::button("Lead")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .test_id("lead")
                })
                .trailing(|| {
                    kivact::ui::button("Trail")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .test_id("trail")
                }),
                ui::property_row(
                    "Plain",
                    kivact::ui::button("Only value").variant(kivact::ui::ButtonVariant::Secondary),
                ),
                ui::property_row(
                    "Wide",
                    kivact::ui::button("Wide value").variant(kivact::ui::ButtonVariant::Secondary),
                )
                .label_width(140.0)
                .row_height(40.0),
                ui::property_row("Short", node().style(style().height(10))).row_height(4.0),
            ))
            .style(style().width(400)),
        )
    });
    let label = screen.get_by_text("Opacity").rect().expect("label");
    let x = |id: &str| screen.get_by_test_id(id).rect().expect(id).x;
    assert!(
        near(label.width, 90.0, 0.6),
        "default label column: {label:?}"
    );
    assert!(label.x + label.width <= x("lead") + 0.5);
    assert!(x("lead") + 0.5 < x("value"));
    assert!(x("value") + 0.5 < x("trail"));
    let row = row_of(&screen, "Opacity");
    let value = screen.get_by_test_id("value").rect().expect("value");
    let trail = screen.get_by_test_id("trail").rect().expect("trail");
    assert!(
        near(trail.x + trail.width, row.x + row.width, 0.6),
        "trailing ends the row"
    );
    assert!(
        value.width > 150.0,
        "the value takes what is left: {value:?}"
    );
    assert!(near(row.height, 24.0, 0.6), "{row:?}");
    assert!(near(width(&screen.get_by_text("Wide")), 140.0, 0.6));
    assert!(near(row_of(&screen, "Wide").height, 40.0, 0.6));
    assert!(
        near(row_of(&screen, "Short").height, 20.0, 0.6),
        "never under 20"
    );
    // A row without slots is the label and the value.
    let only = screen
        .get_by_role(Role::Button, "Only value")
        .rect()
        .expect("only");
    assert!(near(only.x, 90.0 + 4.0, 0.6), "{only:?}");
    clean(&screen);
}
