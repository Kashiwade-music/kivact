//! Role: Tests for CSS Grid layout and properties.
//!
//! CSS Grid (`grid(..)` and the grid style properties): track templates
//! (px, percentages, `fr`, `minmax`, `repeat` with a count and with
//! auto-fill), item placement (spans, line numbers, `1 / -1`), the row
//! and column gaps, auto rows and flow, and alignment, checked on the laid
//! out boxes of real windows (and their resizes).

use kivact::prelude::*;
use kivact::style::pct;
use kivact::style::{GridPlacement, Track};
use kivact::test::render_with;

use crate::layout_units::{assert_rect, rect_of, window};

/// A cell `id` of height `h` (as wide as its track: grid items stretch).
fn cell(id: &'static str, h: i32) -> Node {
    node().test_id(id).style(style().height(h))
}

#[test]
fn twelve_column_grid_places_spans_and_whole_row_items() {
    let screen = render_with(window(1200.0, 600.0), || {
        // grid-template-columns: repeat(12, minmax(0, 1fr))
        grid((
            cell("span4", 30).style(style().grid_column(GridPlacement::span(4))),
            cell("span8", 30).style(style().grid_column(GridPlacement::span(8))),
            // grid-column: 1 / -1
            cell("full", 20).style(style().grid_column((1, -1))),
            // grid-column: 2 / span 3
            cell("from2", 10).style(style().grid_column((2, GridPlacement::span(3)))),
            // grid-column: 11; grid-row: 4 (placed before the others)
            cell("explicit", 5).style(style().grid_column(11).grid_row(4)),
        ))
        .style(
            style()
                .grid_template_columns(Track::repeat(12, Track::minmax(0, Track::fr(1))))
                .row_gap(4),
        )
    });
    assert_rect(rect_of(&screen, "span4"), (0.0, 0.0, 400.0, 30.0), "span 4");
    assert_rect(
        rect_of(&screen, "span8"),
        (400.0, 0.0, 800.0, 30.0),
        "span 8",
    );
    assert_rect(
        rect_of(&screen, "full"),
        (0.0, 34.0, 1200.0, 20.0),
        "1 / -1",
    );
    assert_rect(
        rect_of(&screen, "from2"),
        (100.0, 58.0, 300.0, 10.0),
        "2 / span 3",
    );
    assert_rect(
        rect_of(&screen, "explicit"),
        (1000.0, 72.0, 100.0, 5.0),
        "column 11, row 4",
    );
}

#[test]
fn templates_mix_px_percentages_fr_and_minmax() {
    let screen = render_with(window(800.0, 600.0), || {
        column((
            // .workspace { grid-template-columns: 248px 1fr; gap: 16px }
            grid((cell("side", 10), cell("main", 10)))
                .style(style().grid_template_columns((248, Track::fr(1))).gap(16)),
            // .props { grid-template-columns: minmax(70px, 38%) 1fr 20px }
            grid((cell("label", 10), cell("value", 10), cell("tail", 10))).style(
                style().grid_template_columns((Track::minmax(70, pct(38)), Track::fr(1), 20)),
            ),
            // grid-template-columns: 1fr 56px; a 1.5fr 1fr pair
            grid((cell("grow", 10), cell("fixed", 10)))
                .style(style().grid_template_columns((Track::fr(1), 56))),
            grid((cell("wide", 10), cell("narrow", 10))).style(
                style()
                    .grid_template_columns((Track::fr(1.5), Track::fr(1)))
                    .column_gap(50),
            ),
        ))
    });
    assert_rect(rect_of(&screen, "side"), (0.0, 0.0, 248.0, 10.0), "248px");
    assert_rect(rect_of(&screen, "main"), (264.0, 0.0, 536.0, 10.0), "1fr");
    assert_rect(rect_of(&screen, "label"), (0.0, 10.0, 304.0, 10.0), "38%");
    assert_rect(rect_of(&screen, "value"), (304.0, 10.0, 476.0, 10.0), "1fr");
    assert_rect(rect_of(&screen, "tail"), (780.0, 10.0, 20.0, 10.0), "20px");
    assert_rect(rect_of(&screen, "grow"), (0.0, 20.0, 744.0, 10.0), "1fr");
    assert_rect(rect_of(&screen, "fixed"), (744.0, 20.0, 56.0, 10.0), "56px");
    assert_rect(rect_of(&screen, "wide"), (0.0, 30.0, 450.0, 10.0), "1.5fr");
    assert_rect(
        rect_of(&screen, "narrow"),
        (500.0, 30.0, 300.0, 10.0),
        "1fr",
    );

    // The minmax's percentage falls to its 70 px minimum on a narrow window.
    screen.resize(150.0, 600.0);
    assert_rect(
        rect_of(&screen, "label"),
        (0.0, 10.0, 70.0, 10.0),
        "70px floor",
    );
}
