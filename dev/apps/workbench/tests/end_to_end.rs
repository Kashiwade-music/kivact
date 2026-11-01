//! Role: Workbench end to end on a real Arvane window (`kivact::test`).
#![warn(redundant_imports)]

use kivact::draw::Point;
use kivact::prelude::*;
use kivact::test::{Screen, SemanticNode, render};
use kivact_workbench::{generate_records, workbench};

fn screen() -> Screen {
    render(|| workbench(generate_records(100_000)))
}

fn list_items(n: &SemanticNode) -> usize {
    usize::from(n.role == Some(Role::ListItem)) + n.children.iter().map(list_items).sum::<usize>()
}

#[test]
fn hundred_thousand_keys_materialize_only_the_window() {
    let s = screen();
    assert_eq!(
        s.get_by_test_id("visible-count").text(),
        "100000 visible records / 100000 total"
    );
    let mounted = list_items(&s.snapshot());
    assert!((5..=30).contains(&mounted), "window only: {mounted}");
    assert!(s.get_by_role(Role::List, "Records").exists());
    assert!(s.get_by_role(Role::TextBox, "Search records").exists());
    assert!(s.get_by_role(Role::Canvas, "Record timeline").exists());
    s.get_by_role(Role::ListItem, "Record 0");
}

#[test]
fn external_search_filters_and_row_open_mounts_detail() {
    let s = screen();
    s.get_by_role(Role::TextBox, "Search records")
        .type_text("timeout");
    assert_eq!(
        s.get_by_test_id("visible-count").text(),
        "12500 visible records / 100000 total"
    );
    // Only every 8th record matches: row 8 follows row 0.
    s.get_by_role(Role::ListItem, "Record 8");
    assert!(s.query_by_role(Role::ListItem, "Record 1").is_none());
    s.get_by_test_id("record-open-0").click();
    assert_eq!(
        s.get_by_test_id("detail-body").text(),
        "Record 0: diagnostic detail"
    );
    assert!(s.get_by_role(Role::ListItem, "Record 0").is_selected());
    // The detail follows the selection without remounting.
    s.get_by_test_id("record-open-8").press_key(Key::Enter);
    assert_eq!(
        s.get_by_test_id("detail-body").text(),
        "Record 8: diagnostic detail"
    );
    s.get_by_test_id("clear-selection").click();
    assert!(s.query_by_test_id("detail-body").is_none());
    assert!(s.errors().is_empty(), "{:?}", s.errors());
}

#[test]
fn external_timeline_range_restricts_the_list() {
    let s = screen();
    let t = s.get_by_role(Role::Canvas, "Record timeline");
    let w = t.rect().expect("laid out").width;
    let per = w / 100.0;
    // Buckets 50..=59 = records 50,000..59,999.
    t.drag(Point::new(per * 50.5, 20.0), Point::new(per * 59.5, 20.0));
    assert_eq!(
        s.get_by_test_id("visible-count").text(),
        "10000 visible records / 100000 total"
    );
    assert_eq!(t.value().as_deref(), Some("#50000 – #59000 (10 of 100)"));
    s.get_by_role(Role::ListItem, "Record 50000");
    // Keyboard moves the range; a11y leaves expose the handles.
    t.press_key(Key::ArrowRight);
    s.get_by_role(Role::ListItem, "Record 51000");
    assert_eq!(
        t.get_by_role(Role::Slider, "Selection start")
            .value()
            .as_deref(),
        Some("#51000")
    );
    s.get_by_test_id("clear-range").click();
    assert_eq!(
        s.get_by_test_id("visible-count").text(),
        "100000 visible records / 100000 total"
    );
}

#[test]
fn virtual_list_scrolls_across_hundred_thousand_records() {
    let s = screen();
    let list = s.get_by_role(Role::List, "Records");
    list.scroll(0.0, 50_000.0 * 36.0);
    assert!(s.query_by_test_id("record-open-50000").is_some());
    assert!(s.query_by_test_id("record-open-0").is_none());
    assert!(list_items(&s.snapshot()) <= 30);
    list.scroll(0.0, -50_000.0 * 36.0);
    assert!(s.query_by_test_id("record-open-0").is_some());
    assert_eq!(s.stats().stale_events, 0);
}

#[test]
fn toolbar_icon_buttons_are_named_and_follow_the_state() {
    let s = screen();
    let toolbar = s.get_by_role(Role::Toolbar, "Record tools");
    let tools = s.within(toolbar);
    let clear_selection = tools.get_by_role(Role::Button, "Clear selection");
    let clear_range = tools.get_by_role(Role::Button, "Clear range");
    assert!(clear_selection.is_disabled() && clear_range.is_disabled());
    // Each face is an unnamed (decorative) icon inside its button:
    // assistive technology does not see it.
    assert!(
        s.within(clear_selection)
            .all_by_role(Role::Image, "")
            .is_empty()
    );
    let faces = clear_selection.children();
    assert_eq!(faces.len(), 1);
    assert_eq!(faces[0].role(), None);
    s.get_by_test_id("record-open-0").click();
    assert!(!clear_selection.is_disabled());
    clear_selection.click();
    assert!(s.query_by_test_id("detail-body").is_none());
    assert!(clear_selection.is_disabled());
    assert!(s.errors().is_empty(), "{:?}", s.errors());
}
