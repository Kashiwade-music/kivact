//! Role: Tests for SearchBox, Timeline, and LazyList components.
//!
//! External components: SearchBox (IME), Timeline (pointer, hit regions, semantics) and
//! LazyList (own virtualization), each also placed twice on one screen.

use kivact::primitive::NodeExt;
use std::cell::RefCell;
use std::rc::Rc;

use kivact::Binding;
use kivact::draw::Point;
use kivact::prelude::*;
use kivact::test::{Screen, render};
use kivact::ui::ListItem;
use kivact::view::{A11yAction, callback};
use kivact_external_component::{Span, lazy_list, search_box, timeline};

fn mount<H: 'static, V: IntoView>(f: impl FnOnce() -> (H, V) + 'static) -> (Screen, H) {
    let slot: Rc<RefCell<Option<H>>> = Rc::new(RefCell::new(None));
    let s2 = slot.clone();
    let screen = render(move || {
        let (h, v) = f();
        *s2.borrow_mut() = Some(h);
        v
    });
    let h = slot.borrow_mut().take().expect("setup ran");
    (screen, h)
}

// ---------------------------------------------------------------- SearchBox

#[test]
fn search_box_commits_ime_text_only_and_clears() {
    let submitted = Rc::new(RefCell::new(Vec::new()));
    let sub = submitted.clone();
    let (screen, query) = mount(move || {
        let query = signal(String::new());
        let on_submit = callback(move |q| sub.borrow_mut().push(q));
        let hits = memo(move || format!("{} matches", query.with(|q| q.chars().count())));
        (
            query,
            search_box("Search logs", query)
                .placeholder("type to filter")
                .status(hits)
                .on_submit(on_submit),
        )
    });
    let input = screen.get_by_role(Role::TextBox, "Search logs");
    assert!(screen.query_by_role(Role::Button, "Clear search").is_none());
    input.compose("けんさく");
    assert_eq!(
        query.get_untracked(),
        "",
        "preedit never reaches the binding"
    );
    input.type_text("検索");
    assert_eq!(query.get_untracked(), "検索");
    assert_eq!(input.value().as_deref(), Some("検索"));
    assert_eq!(
        screen.get_by_role(Role::Status, "2 matches").text(),
        "2 matches"
    );
    input.press_key(Key::Enter);
    assert_eq!(*submitted.borrow(), vec!["検索".to_string()]);
    // Clear button appears with text and clears it.
    screen.get_by_role(Role::Button, "Clear search").click();
    assert_eq!(query.get_untracked(), "");
    assert!(screen.query_by_role(Role::Button, "Clear search").is_none());
    // Escape clears too (bubbles from the input to the search box).
    input.type_text("abc");
    input.press_key(Key::Escape);
    assert_eq!(query.get_untracked(), "");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn search_box_follows_external_writes() {
    let (screen, query) = mount(|| {
        let query = signal(String::from("start"));
        (query, search_box("Find", query))
    });
    let input = screen.get_by_role(Role::TextBox, "Find");
    assert_eq!(input.value().as_deref(), Some("start"));
    query.set("from code".into());
    screen.run_until_idle();
    assert_eq!(input.value().as_deref(), Some("from code"));
}

// ----------------------------------------------------------------- Timeline

fn series() -> Vec<f32> {
    (0..20).map(|i| ((i * 7) % 11) as f32).collect()
}

fn timeline_screen() -> (Screen, Signal<Option<Span>>) {
    mount(|| {
        let sel = signal(None::<Span>);
        let data = signal(series());
        (
            sel,
            column((
                timeline("Requests", data, sel)
                    .bucket_label(|i| format!("{:02}:00", i))
                    .test_id("tl"),
                text(move || format!("{:?}", sel.get())),
            ))
            .style(style().width(400)),
        )
    })
}

#[test]
fn timeline_drag_selects_a_range_and_regions_move_handles() {
    let (screen, sel) = timeline_screen();
    let t = screen.get_by_role(Role::Canvas, "Requests");
    let r = t.rect().expect("laid out");
    assert_eq!(r.width, 400.0);
    assert_eq!(t.value().as_deref(), Some("no selection (20 buckets)"));
    // 20 buckets over 400 px: 20 px per bucket.
    t.drag(Point::new(45.0, 30.0), Point::new(125.0, 30.0));
    assert_eq!(sel.get_untracked(), Some(Span { start: 2, end: 7 }));
    assert_eq!(t.value().as_deref(), Some("02:00 – 06:00 (5 of 20)"));
    // Regions are exposed as semantic leaves of the canvas.
    let end = t.get_by_role(Role::Slider, "Selection end");
    assert_eq!(end.value().as_deref(), Some("06:00"));
    assert_eq!(
        t.get_by_role(Role::Group, "Selected range")
            .value()
            .as_deref(),
        Some("02:00 – 06:00")
    );
    // Dragging the end handle (region hit) resizes; the start stays.
    end.drag(Point::new(140.0, 30.0), Point::new(305.0, 30.0));
    assert_eq!(sel.get_untracked(), Some(Span { start: 2, end: 16 }));
    // Dragging the start handle past the end flips the range.
    t.get_by_role(Role::Slider, "Selection start")
        .drag(Point::new(40.0, 30.0), Point::new(390.0, 30.0));
    assert_eq!(sel.get_untracked(), Some(Span { start: 15, end: 20 }));
    // Dragging the selected span moves it, keeping its length.
    t.get_by_role(Role::Group, "Selected range")
        .drag(Point::new(330.0, 30.0), Point::new(130.0, 30.0));
    assert_eq!(sel.get_untracked(), Some(Span { start: 5, end: 10 }));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn timeline_keyboard_and_a11y_actions() {
    let (screen, sel) = timeline_screen();
    let t = screen.get_by_test_id("tl");
    t.press_key(Key::ArrowRight);
    assert_eq!(sel.get_untracked(), Some(Span { start: 0, end: 1 }));
    t.press_key(Key::ArrowRight);
    assert_eq!(sel.get_untracked(), Some(Span { start: 1, end: 2 }));
    t.press_key(Key::End);
    assert_eq!(sel.get_untracked(), Some(Span { start: 19, end: 20 }));
    t.a11y(A11yAction::Decrement);
    assert_eq!(sel.get_untracked(), Some(Span { start: 18, end: 19 }));
    t.a11y(A11yAction::Increment);
    t.a11y(A11yAction::Increment);
    assert_eq!(
        sel.get_untracked(),
        Some(Span { start: 19, end: 20 }),
        "clamped"
    );
    t.press_key(Key::Home);
    assert_eq!(sel.get_untracked(), Some(Span { start: 0, end: 1 }));
    t.press_key(Key::Escape);
    assert_eq!(sel.get_untracked(), None);
    assert!(
        screen
            .query_by_role(Role::Slider, "Selection start")
            .is_none()
    );
}

// ----------------------------------------------------------------- LazyList

#[derive(Clone, PartialEq, Debug)]
struct Item {
    id: u32,
    label: String,
}

fn items(n: u32) -> Vec<Item> {
    (0..n)
        .map(|id| Item {
            id,
            label: format!("item {id}"),
        })
        .collect()
}

struct ListH {
    all: Signal<Vec<Item>>,
    selected: Signal<Option<u32>>,
    opened: Signal<Option<u32>>,
}

fn list_screen(n: u32) -> (Screen, ListH) {
    mount(move || {
        let all = signal(items(n));
        let selected = signal(None::<u32>);
        let opened = signal(None::<u32>);
        let open = callback(move |id| opened.set(Some(id)));
        (
            ListH {
                all,
                selected,
                opened,
            },
            lazy_list(
                "Items",
                all,
                |it: &Item| it.id,
                |_, it: Value<Item>| text(move || it.with(|i| i.label.clone())),
            )
            .row_height(20.0)
            .selection(selected)
            .on_activate(open)
            .item_name(|it: &Item| format!("row {}", it.id))
            .style(style().height(200))
            .test_id("list"),
        )
    })
}

fn mounted_rows(screen: &Screen) -> usize {
    count_items(&screen.snapshot())
}

fn count_items(n: &kivact::test::SemanticNode) -> usize {
    usize::from(n.role == Some(Role::ListItem)) + n.children.iter().map(count_items).sum::<usize>()
}

#[test]
fn lazy_list_mounts_only_the_window_of_100k_items() {
    let (screen, h) = list_screen(100_000);
    let list = screen.get_by_role(Role::List, "Items");
    assert_eq!(list.value().as_deref(), Some("100000 items"));
    let first = mounted_rows(&screen);
    assert!((10..=12).contains(&first), "200 px / 20 px + 1: {first}");
    screen.get_by_text("item 0");
    list.scroll(0.0, 20_000.0);
    screen.get_by_text("item 1000");
    assert!(screen.query_by_text("item 0").is_none());
    assert!(mounted_rows(&screen) <= 12);
    // The scrollbar thumb is a region with a value.
    assert_eq!(
        list.get_by_role(Role::Slider, "Scroll position")
            .value()
            .as_deref(),
        Some("row 1001 of 100000")
    );
    // Filtering shrinks the content: the offset clamps, rows stay visible.
    h.all.set(items(30));
    screen.run_until_idle();
    screen.get_by_text("item 29");
    assert!(mounted_rows(&screen) >= 10);
    let s = screen.stats();
    assert_eq!(s.stale_events, 0);
}

#[test]
fn lazy_list_keyboard_click_and_activation() {
    let (screen, h) = list_screen(1_000);
    let list = screen.get_by_test_id("list");
    list.press_key(Key::ArrowDown);
    assert_eq!(h.selected.get_untracked(), Some(0));
    assert!(screen.get_by_role(Role::ListItem, "row 0").is_selected());
    list.press_key(Key::End);
    assert_eq!(h.selected.get_untracked(), Some(999));
    // End reveals the last row.
    assert!(screen.get_by_role(Role::ListItem, "row 999").is_selected());
    assert!(screen.query_by_text("item 0").is_none());
    list.press_key(Key::PageUp);
    let s = h.selected.get_untracked().expect("selected");
    assert!((985..999).contains(&s), "{s}");
    list.press_key(Key::Enter);
    assert_eq!(h.opened.get_untracked(), Some(s));
    list.press_key(Key::Home);
    screen.get_by_role(Role::ListItem, "row 3").click();
    assert_eq!(h.selected.get_untracked(), Some(3));
    assert_eq!(list.value().as_deref(), Some("1000 items, row 4 selected"));
    // Only the two affected rows changed selection state.
    assert!(!screen.get_by_role(Role::ListItem, "row 0").is_selected());
}

#[test]
fn lazy_list_root_takes_the_semantic_selected_state() {
    // `selection` (the selected row) does not shadow `NodeBuilder::selected`.
    let (screen, ()) = mount(|| {
        let picked = signal(Some(1u32));
        (
            (),
            lazy_list(
                "Picked",
                signal(items(5)),
                |it: &Item| it.id,
                |_, it: Value<Item>| text(move || it.with(|i| i.label.clone())),
            )
            .selection(picked)
            .selected(true)
            .item_name(|it: &Item| format!("row {}", it.id))
            .style(style().height(200)),
        )
    });
    assert!(screen.get_by_role(Role::List, "Picked").is_selected());
    assert!(screen.get_by_role(Role::ListItem, "row 1").is_selected());
}

#[test]
fn lazy_list_scrollbar_thumb_drags() {
    let (screen, _h) = list_screen(1_000);
    let thumb = screen.get_by_role(Role::Slider, "Scroll position");
    assert_eq!(thumb.value().as_deref(), Some("row 1 of 1000"));
    thumb.drag(Point::new(795.0, 10.0), Point::new(795.0, 110.0));
    let v = screen
        .get_by_role(Role::Slider, "Scroll position")
        .value()
        .unwrap_or_default();
    assert!(v.starts_with("row ") && v != "row 1 of 1000", "{v}");
    assert!(screen.query_by_text("item 0").is_none());
}

// ------------------------------------------------------------------- twins

fn pane() -> impl IntoView {
    let q = signal(String::new());
    let sel = signal(None::<Span>);
    let data = signal(series());
    let all = signal(items(500));
    column((
        search_box("Search", q),
        text(move || format!("q={}", q.get())).test_id("q"),
        timeline("Volume", data, sel),
        text(move || format!("{:?}", sel.get())).test_id("sel"),
        lazy_list(
            "Rows",
            all,
            |i: &Item| i.id,
            |_, it: Value<Item>| text(move || it.with(|i| i.label.clone())),
        )
        .row_height(20.0)
        .style(style().height(100)),
    ))
    .style(style().width(380))
}

#[test]
fn the_same_components_twice_on_one_screen_do_not_collide() {
    let screen = render(|| {
        row((
            node().test_id("left").child(pane()),
            node().test_id("right").child(pane()),
        ))
    });
    // Same names in both panes: a global query is reported as ambiguous.
    let global = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        screen.get_by_role(Role::TextBox, "Search");
    }));
    assert!(
        global.is_err(),
        "ambiguity is reported, not silently resolved"
    );
    let left = screen.within(screen.get_by_test_id("left"));
    let right = screen.within(screen.get_by_test_id("right"));
    left.get_by_role(Role::TextBox, "Search").type_text("alpha");
    right.get_by_role(Role::TextBox, "Search").type_text("beta");
    assert_eq!(left.get_by_test_id("q").text(), "q=alpha");
    assert_eq!(right.get_by_test_id("q").text(), "q=beta");
    left.get_by_role(Role::Canvas, "Volume")
        .drag(Point::new(5.0, 10.0), Point::new(60.0, 10.0));
    assert_eq!(
        left.get_by_test_id("sel").text(),
        "Some(Span { start: 0, end: 4 })"
    );
    assert_eq!(right.get_by_test_id("sel").text(), "None");
    left.get_by_role(Role::List, "Rows").scroll(0.0, 2_000.0);
    assert!(left.query_by_text("item 0").is_none());
    assert!(right.query_by_text("item 0").is_some());
    // Region ids are local to each canvas: both expose region 1..3 without clash.
    right
        .get_by_role(Role::Canvas, "Volume")
        .press_key(Key::ArrowRight);
    assert_eq!(
        right.get_by_test_id("sel").text(),
        "Some(Span { start: 0, end: 1 })"
    );
    assert_eq!(screen.all_by_role(Role::Slider, "Selection start").len(), 2);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

// ---------------------------------------------------------------- locations

/// Every located node under `n` whose file is not this test file.
fn located_elsewhere(n: &kivact::test::SemanticNode, out: &mut Vec<String>) {
    if let Some(at) = n.at
        && !at.file().ends_with("components.rs")
    {
        out.push(format!("{:?} {:?} @{at}", n.role, n.name));
    }
    for c in &n.children {
        located_elsewhere(c, out);
    }
}

/// The first node under `n` with `role` and `name`.
fn find<'a>(
    n: &'a kivact::test::SemanticNode,
    role: Role,
    name: &str,
) -> Option<&'a kivact::test::SemanticNode> {
    if n.role == Some(role) && n.name == name {
        return Some(n);
    }
    n.children.iter().find_map(|c| find(c, role, name))
}

fn choice_options() -> Vec<ListItem<&'static str>> {
    vec![ListItem::new("x", "x"), ListItem::new("y", "y")]
}

#[test]
fn every_instance_is_located_at_its_own_call_site() {
    use kivact_external_component::parts::{CROSS_ICON, icon_button};
    let screen = render(|| {
        let (q, sel, on) = (signal(String::new()), signal(None::<Span>), signal(false));
        let (data, all) = (signal(series()), signal(items(50)));
        let pick = signal(None::<&'static str>);
        column((
            {
                let label = ("A").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
                    .style(
                        style()
                            .padding(8)
                            .min_height(32)
                            .background(ColorToken::SurfaceAlt)
                            .focus(style().ring(2, ColorToken::Focus)),
                    )
            },
            {
                let label = ("B").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
                    .style(
                        style()
                            .padding(8)
                            .min_height(32)
                            .background(ColorToken::SurfaceAlt)
                            .focus(style().ring(2, ColorToken::Focus)),
                    )
            },
            icon_button("Close", CROSS_ICON),
            {
                let checked: Binding<bool> = (on).into();
                node()
                    .role(Role::CheckBox)
                    .focusable(true)
                    .name("Flag")
                    .checked(checked)
                    .on_press(move || checked.set(!checked.get_untracked()))
                    .child(text(move || {
                        (if checked.get() { "[x]" } else { "[ ]" }).to_owned()
                    }))
                    .child(text("Flag"))
                    .style(
                        style()
                            .padding(8)
                            .min_height(32)
                            .focus(style().ring(2, ColorToken::Focus)),
                    )
            },
            {
                let options = choice_options();
                let group = kivact::interaction::use_list(
                    options.clone(),
                    kivact::interaction::ListSelection::Single(pick.into()),
                )
                .policy(kivact::interaction::ListPolicy::radio());
                let members = options
                    .into_iter()
                    .map(|option| {
                        let label = option.label();
                        node()
                            .part(
                                group
                                    .item(*option.key())
                                    .role(Role::Option)
                                    .name(label.clone()),
                            )
                            .child(text(label))
                    })
                    .collect::<Vec<_>>();
                column(members)
                    .part(group.list().role(Role::ListBox))
                    .style(style().gap(4))
            }
            .name("Pick"),
            search_box("Find", q),
            timeline("Volume", data, sel),
            lazy_list(
                "Rows",
                all,
                |i: &Item| i.id,
                |_, it: Value<Item>| text(move || it.with(|i| i.label.clone())),
            )
            .row_height(20.0)
            .style(style().height(100)),
        ))
    });
    let snap = screen.snapshot();
    let line = |role, name| find(&snap, role, name).and_then(|n| n.at).map(|a| a.line());
    assert!(line(Role::Button, "A").is_some());
    assert!(line(Role::Button, "B").is_some());
    assert!(line(Role::Button, "Close").is_some());
    assert!(line(Role::CheckBox, "Flag").is_some());
    assert!(line(Role::ListBox, "Pick").is_some());
    assert!(line(Role::TextBox, "Find").is_some());
    assert!(line(Role::Canvas, "Volume").is_some());
    assert!(line(Role::List, "Rows").is_some());
    // Choice members are present directly in the primitive list box.
    let mut elsewhere = Vec::new();
    located_elsewhere(&screen.snapshot(), &mut elsewhere);
    assert!(
        elsewhere.is_empty(),
        "located inside a component: {elsewhere:#?}"
    );
}
