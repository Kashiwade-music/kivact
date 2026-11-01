//! Role: Tests for the device status monitor app of the external components.
//!
//! Device status monitor (second external-component app, skin switching):
//! live updates, filters, lazy list, two timelines with range statistics,
//! skins.
#![warn(redundant_imports)]

use std::time::Duration;

use kivact::draw::Point;
use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen, SemanticNode, render_with};
use kivact_ext_apps::device_monitor::{Status, app, fleet};
use kivact_external_component::skin::Skin;

fn screen(skin: Skin, tick_ms: Option<u64>) -> Screen {
    render_with(
        RenderOptions {
            theme: skin.theme(),
            width: 1100.0,
            height: 760.0,
            ..RenderOptions::from_env()
        },
        move || app(fleet(5, 1_000), skin, tick_ms),
    )
}

fn strip(mut n: SemanticNode) -> SemanticNode {
    if n.name.starts_with("Skin: ") {
        n.name.clear();
    }
    n.focused = false;
    n.children = n.children.into_iter().map(strip).collect();
    n
}

#[test]
fn filters_and_windowed_list_over_1000_devices() {
    let s = screen(Skin::Business, None);
    let fleet = fleet(5, 1_000);
    let list = s.get_by_role(Role::List, "Devices");
    assert_eq!(list.value().as_deref(), Some("1000 items, row 1 selected"));
    let items = |s: &Screen| {
        fn count(n: &SemanticNode) -> usize {
            usize::from(n.role == Some(Role::ListItem))
                + n.children.iter().map(count).sum::<usize>()
        }
        count(&s.snapshot())
    };
    assert!(
        items(&s) < 40,
        "only visible rows are mounted: {}",
        items(&s)
    );
    s.get_by_role(Role::CheckBox, "Alarms only").click();
    let alarms = fleet
        .iter()
        .filter(|d| matches!(d.status, Status::Alarm | Status::Offline))
        .count();
    assert!(alarms > 0);
    s.get_by_text(&format!("{alarms} shown"));
    s.get_by_role(Role::CheckBox, "Alarms only").click();
    s.get_by_role(Role::TextBox, "Find device")
        .type_text("roof");
    s.get_by_text("250 shown");
    s.get_by_role(Role::Button, "Clear search").click();
    s.get_by_role(Role::TextBox, "Find device")
        .type_text("sensor-0042");
    s.get_by_text("1 shown");
    s.get_by_role(Role::ListItem, "device 42").click();
    s.get_by_text(&format!(
        "sensor-0042 · Roof · {}",
        fleet[42].status.label()
    ));
    assert!(s.errors().is_empty(), "{:?}", s.errors());
}

#[test]
fn two_timelines_give_independent_range_statistics() {
    let s = screen(Skin::Business, None);
    let d0 = &fleet(5, 1)[0];
    let temp = s.get_by_role(Role::Canvas, "Temperature history");
    let volt = s.get_by_role(Role::Canvas, "Voltage history");
    let full = s.get_by_test_id("temp-stats").text();
    assert!(full.starts_with("48 samples"), "{full}");
    let w = temp.rect().expect("laid out").width;
    let per = w / 48.0;
    temp.drag(Point::new(per * 40.5, 30.0), Point::new(per * 47.5, 30.0));
    let part = &d0.temp[40..48];
    let avg = part.iter().sum::<f32>() / 8.0;
    let got = s.get_by_test_id("temp-stats").text();
    assert!(got.starts_with("8 samples"), "{got}");
    assert!(got.contains(&format!("avg {avg:.1} °C")), "{got} vs {avg}");
    assert!(
        s.get_by_test_id("volt-stats")
            .text()
            .starts_with("48 samples")
    );
    volt.press_key(Key::ArrowRight);
    assert!(
        s.get_by_test_id("volt-stats")
            .text()
            .starts_with("1 samples")
    );
    assert!(
        s.get_by_test_id("temp-stats")
            .text()
            .starts_with("8 samples")
    );
    assert_eq!(temp.value().as_deref(), Some("t-8 – t-1 (8 of 48)"));
}

#[test]
fn live_ticks_update_rows_and_detail() {
    let s = screen(Skin::Business, Some(1_000));
    let first = s.get_by_role(Role::Canvas, "Temperature history").value();
    s.advance(Duration::from_millis(3_100));
    let after = s.get_by_role(Role::Canvas, "Temperature history").value();
    assert_eq!(first, after, "selection semantics are stable across ticks");
    let stats = s.stats();
    s.advance(Duration::from_millis(1_000));
    let next = s.stats();
    assert!(
        next.set_props > stats.set_props,
        "the tick repainted something"
    );
    assert_eq!(
        next.creates, stats.creates,
        "a tick re-renders, it does not remount"
    );
}

#[test]
fn business_and_console_skins_have_identical_semantics_and_hit_regions() {
    let a = screen(Skin::Business, None);
    let b = screen(Skin::Console, None);
    assert_eq!(strip(a.snapshot()), strip(b.snapshot()));
    let size = |s: &Screen, name: &str| {
        s.get_by_role(Role::Canvas, name)
            .rect()
            .map(|r| (r.width, r.height))
    };
    assert_eq!(
        size(&a, "Temperature history"),
        size(&b, "Temperature history")
    );
    for s in [&a, &b] {
        let t = s.get_by_role(Role::Canvas, "Temperature history");
        t.drag(Point::new(30.0, 40.0), Point::new(200.0, 40.0));
        t.get_by_role(Role::Slider, "Selection start")
            .drag(Point::new(30.0, 40.0), Point::new(80.0, 40.0));
        s.get_by_role(Role::List, "Devices")
            .press_key(Key::ArrowDown);
        assert!(s.errors().is_empty(), "{:?}", s.errors());
    }
    assert_eq!(strip(a.snapshot()), strip(b.snapshot()));
    // Runtime switch: paint only.
    let creates = a.stats().creates;
    a.get_by_role(Role::Button, "Skin: Business").click();
    a.get_by_role(Role::Button, "Skin: Console");

    assert_eq!(a.stats().creates, creates);
    assert_eq!(strip(a.snapshot()), strip(b.snapshot()));
}
