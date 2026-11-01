//! Role: Tests for the log viewer app of the external components.
//!
//! Log viewer (first external-component app): search, level filter, timeline range, lazy
//! list, detail; two panes with the same components; live tail.
#![warn(redundant_imports)]

use std::time::Duration;

use kivact::draw::Point;
use kivact::prelude::*;
use kivact::test::{Screen, render};
use kivact_ext_apps::log_viewer::{BUCKET_MINUTES, Level, LogViewerOptions, app, generate};

fn options(live: bool) -> LogViewerOptions {
    LogViewerOptions {
        left: generate(7, 6_000, 95),
        right: generate(11, 2_000, 180),
        live,
        ..LogViewerOptions::default()
    }
}

fn screen(live: bool) -> Screen {
    let o = options(live);
    render(move || app(o))
}

#[test]
fn search_level_and_time_range_filter_one_pane_only() {
    let s = screen(false);
    let panes = s.all_by_test_id("search");
    assert_eq!(panes.len(), 2, "two search boxes");
    let left_lines = generate(7, 6_000, 95);
    let left = s.within(panes[0]);
    let status = || left.get_by_test_id("status").text();
    assert_eq!(status(), "6000 of 6000 lines");
    left.get_by_role(Role::TextBox, "Search")
        .type_text("timeout");
    let expect = left_lines.iter().filter(|l| l.matches("timeout")).count();
    assert_eq!(status(), format!("{expect} of 6000 lines"));
    // The other pane is untouched.
    let right = s.within(s.all_by_test_id("search")[1]);
    assert_eq!(right.get_by_test_id("status").text(), "2000 of 2000 lines");
    // Level filter through the skinned standard select.
    let lvl = s.all_by_role(Role::ComboBox, "Level");
    assert_eq!(lvl.len(), 2);
    lvl[0].select_option("Errors only");
    let expect = left_lines
        .iter()
        .filter(|l| l.matches("timeout") && l.level == Level::Error)
        .count();
    assert_eq!(status(), format!("{expect} of 6000 lines"));
    // Timeline drag restricts to a time range (5 minutes per bucket).
    let tls = s.all_by_role(Role::Canvas, "Lines per 5 minutes");
    let tl = tls[0];
    let w = tl.rect().expect("laid out").width;
    let bucket_w = w / (240 / BUCKET_MINUTES) as f32;
    tl.drag(
        Point::new(bucket_w * 16.5, 20.0),
        Point::new(bucket_w * 21.5, 20.0),
    );
    let in_range = |m: u16| (16..22).contains(&((m / BUCKET_MINUTES) as usize));
    let expect = left_lines
        .iter()
        .filter(|l| l.matches("timeout") && l.level == Level::Error && in_range(l.minute))
        .count();
    assert!(expect > 0, "the incident is inside the range");
    assert_eq!(status(), format!("{expect} of 6000 lines"));
    assert_eq!(
        tl.value().as_deref(),
        Some("09:20 – 09:45 (6 of 48)"),
        "range semantics name the clock times"
    );
    s.get_by_role(Role::Button, "Clear range").click();
    assert!(s.query_by_role(Role::Button, "Clear range").is_none());
    assert!(s.errors().is_empty(), "{:?}", s.errors());
}

#[test]
fn list_selection_opens_detail() {
    let s = screen(false);
    let lists = s.all_by_role(Role::List, "Log lines");
    assert_eq!(lists.len(), 2);
    let list = lists[1];
    list.press_key(Key::ArrowDown);
    list.press_key(Key::ArrowDown);
    list.press_key(Key::Enter);
    let right = generate(11, 2_000, 180);
    let l = &right[1];
    let details = s.all_by_test_id("detail");
    assert_eq!(
        details[1].text(),
        format!(
            "#1 {} [{}] {}: {}",
            l.clock(),
            l.level.tag(),
            l.source,
            l.message
        )
    );
    assert_eq!(details[0].text(), "Select a line (click, ↑/↓, Enter)");
    // Click on a visible row of the left pane.
    s.all_by_role(Role::ListItem, "line 3")[0].click();
    list.press_key(Key::End);
    assert!(s.query_by_role(Role::ListItem, "line 1999").is_some());
}

#[test]
fn live_tail_appends_to_the_right_pane() {
    let s = screen(true);
    let right = s.within(s.all_by_test_id("search")[1]);
    assert_eq!(right.get_by_test_id("status").text(), "2000 of 2000 lines");
    s.advance(Duration::from_millis(2_200));
    assert_eq!(right.get_by_test_id("status").text(), "2003 of 2003 lines");
    let left = s.within(s.all_by_test_id("search")[0]);
    assert_eq!(left.get_by_test_id("status").text(), "6000 of 6000 lines");
}

#[test]
fn skin_switch_keeps_log_viewer_semantics() {
    let s = screen(false);
    let before = s.snapshot();
    let creates = s.stats().creates;
    s.get_by_role(Role::Button, "Skin: Business").click();
    s.get_by_role(Role::Button, "Skin: Console");
    assert_eq!(s.stats().creates, creates);
    fn strip(mut n: kivact::test::SemanticNode) -> kivact::test::SemanticNode {
        if n.name.starts_with("Skin: ") {
            n.name.clear();
        }
        n.focused = false;
        n.children = n.children.into_iter().map(strip).collect();
        n
    }
    assert_eq!(strip(before), strip(s.snapshot()));
}
