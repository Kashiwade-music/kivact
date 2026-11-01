//! Role: Tests for the outliner on the core `Tree` through the public input.
//!
//! The outliner on the core `Tree`: keys, rename, activation, drops, the
//! active item, trailing cells and filtering, through the public input only.

use super::*;
use kivact::draw::Point;
use kivact::view::callback;

fn scene() -> Vec<ui::TreeItem> {
    vec![
        ui::TreeItem::new(1, None, "Scene", "folder"),
        ui::TreeItem::new(2, Some(1), "Group", "folder"),
        ui::TreeItem::new(3, Some(2), "Cube", "box"),
        ui::TreeItem::new(4, None, "Other", "folder"),
        ui::TreeItem::new(5, Some(1), "Lamp", "bulb"),
    ]
}

fn item<'a>(screen: &'a Screen, name: &str) -> kivact::test::Element<'a> {
    screen.get_by_role(Role::TreeItem, name)
}

/// Drag the row `from` to the point `fraction` of the way down the row
/// `over` (0 is its top edge), as a pointer does.
fn drag_over(screen: &Screen, from: &str, over: &str, fraction: f32) {
    // Presses at one place in a moment are a double click.
    screen.advance(Duration::from_secs(1));
    let a = item(screen, from).rect().expect("dragged row");
    let b = item(screen, over).rect().expect("row under the pointer");
    item(screen, from).drag(
        Point::new(100.0, a.height / 2.0),
        Point::new(100.0, b.y + b.height * fraction - a.y),
    );
}

fn recorded() -> (Signal<Vec<ui::TreeDrop>>, Callback<ui::TreeDrop>) {
    let drops = signal(Vec::new());
    (
        drops,
        callback(move |drop| drops.update(|all| all.push(drop))),
    )
}

#[test]
fn the_rows_follow_the_tree_view_keys_and_are_one_tab_stop() {
    let (screen, (_data, selected)) = kivact::test::render_with_handle(crate::options(), || {
        let data = signal(scene());
        let selected = signal(Vec::new());
        (
            (data, selected),
            column((
                kivact::ui::button("before").variant(kivact::ui::ButtonVariant::Secondary),
                app_outliner(
                    data,
                    ui::OutlinerState {
                        selection: selected.into(),
                        ..ui::OutlinerState::new(ids(data))
                    },
                ),
            )),
        )
    });
    assert!(
        screen
            .get_by_role(Role::Tree, "Outliner")
            .is_multiselectable()
    );
    assert_eq!(item(&screen, "Cube").level(), Some(3));
    assert_eq!(item(&screen, "Lamp").pos_in_set(), Some(2));
    screen.get_by_role(Role::Button, "before").click();
    screen.press_tab(false);
    assert!(item(&screen, "Scene").is_focused(), "one tab stop");
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    assert!(item(&screen, "Cube").is_focused());
    assert_eq!(selected.get_untracked(), [3]);
    screen.press_key(Key::ArrowLeft);
    assert!(
        item(&screen, "Group").is_focused(),
        "a leaf goes to its parent"
    );
    screen.press_key(Key::ArrowLeft);
    assert_eq!(item(&screen, "Group").is_expanded(), Some(false));
    assert!(screen.query_by_role(Role::TreeItem, "Cube").is_none());
    screen.press_key(Key::ArrowRight);
    assert_eq!(item(&screen, "Group").is_expanded(), Some(true));
    screen.press_key(Key::End);
    assert!(item(&screen, "Other").is_focused());
    screen.press_key(Key::Home);
    screen.press_key(Key::Char('*'));
    assert_eq!(item(&screen, "Scene").is_expanded(), Some(true));
    screen.press_key(Key::Char('o'));
    assert!(item(&screen, "Other").is_focused(), "typeahead");
    clean(&screen);
}

#[test]
fn range_toggle_and_restriction_controls_keep_the_expected_selection() {
    let (screen, (data, selection)) = kivact::test::render_with_handle(crate::options(), || {
        let data = signal(vec![
            ui::TreeItem::new(1, None, "A", "folder"),
            ui::TreeItem::new(2, None, "B", "folder"),
            ui::TreeItem::new(3, None, "C", "folder"),
            ui::TreeItem::new(4, Some(3), "D", "folder"),
        ]);
        let selection = signal(Vec::new());
        (
            (data, selection),
            app_outliner(
                data,
                ui::OutlinerState {
                    selection: selection.into(),
                    ..ui::OutlinerState::new(ids(data))
                },
            ),
        )
    });
    screen.get_by_role(Role::TreeItem, "A").click();
    let ctrl = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    screen.key_down(Key::Unidentified, KeyCode::ControlLeft, ctrl);
    screen.get_by_role(Role::TreeItem, "B").click();
    assert_eq!(selection.get_untracked(), vec![1, 2]);
    screen.get_by_role(Role::TreeItem, "B").click();
    assert_eq!(selection.get_untracked(), vec![1]);
    screen.key_up(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers::default(),
    );
    screen.get_by_role(Role::TreeItem, "A").click();
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    screen.key_down(Key::Unidentified, KeyCode::ShiftLeft, shift);
    screen.get_by_role(Role::TreeItem, "C").click();
    assert_eq!(selection.get_untracked(), vec![1, 2, 3]);
    screen.key_up(Key::Unidentified, KeyCode::ShiftLeft, Modifiers::default());
    screen.get_by_role(Role::TreeItem, "A").click();
    screen.press_key_with(Key::ArrowDown, shift);
    screen.press_key_with(Key::ArrowDown, shift);
    assert_eq!(selection.get_untracked(), vec![1, 2, 3]);
    screen.get_by_role(Role::TreeItem, "A").click();
    let visibility = screen.all_by_role(Role::Button, "Tree visibility");
    visibility[1].click();
    assert!(!data.get_untracked()[1].visible);
    assert_eq!(selection.get_untracked(), vec![1]);
    let expand = screen.all_by_role(Role::Button, "Expand tree item");
    assert_eq!(expand.len(), 1, "only a parent has the button");
    expand[0].click();
    assert_eq!(item(&screen, "C").is_expanded(), Some(false));
    assert_eq!(selection.get_untracked(), vec![1]);
    clean(&screen);
}

#[test]
fn shift_subtree_and_restriction_paint_keep_the_row_selection() {
    let (screen, (items, selection)) = kivact::test::render_with_handle(crate::options(), || {
        let items = signal(vec![
            ui::TreeItem::new(1, None, "Root", "folder"),
            ui::TreeItem::new(2, Some(1), "Child", "box"),
            ui::TreeItem::new(3, Some(2), "Leaf", "camera"),
        ]);
        let selection = signal(vec![1]);
        (
            (items, selection),
            app_outliner(
                items,
                ui::OutlinerState {
                    selection: selection.into(),
                    ..ui::OutlinerState::new(ids(items))
                },
            ),
        )
    });
    let shift = Modifiers {
        shift: true,
        ..Default::default()
    };
    for expanded in [false, true] {
        screen.key_down(Key::Unidentified, KeyCode::ShiftLeft, shift);
        item(&screen, "Root")
            .get_by_role(Role::Button, "Expand tree item")
            .click();
        screen.key_up(Key::Unidentified, KeyCode::ShiftLeft, Modifiers::default());
        assert_eq!(item(&screen, "Root").is_expanded(), Some(expanded));
    }
    let flag = item(&screen, "Child").get_by_role(Role::Button, "Tree visibility");
    let a = flag.rect().expect("child flag");
    let b = item(&screen, "Leaf")
        .get_by_role(Role::Button, "Tree visibility")
        .rect()
        .expect("leaf flag");
    flag.drag(
        Point::new(a.width / 2.0, a.height / 2.0),
        Point::new(b.center().x - a.x, b.center().y - a.y),
    );
    let all = items.get_untracked();
    assert!(all[0].visible && !all[1].visible && !all[2].visible);
    assert_eq!(selection.get_untracked(), vec![1]);
    item(&screen, "Leaf").click();
    screen.press_key(Key::ArrowLeft);
    assert_eq!(selection.get_untracked(), vec![2]);
    assert!(item(&screen, "Child").is_focused());
    screen.press_key(Key::Char('h'));
    assert!(items.get_untracked()[1].visible, "H toggles visibility");
    clean(&screen);
}

#[test]
fn a_filter_keeps_the_ancestors_and_leaves_the_expanded_flags() {
    let (screen, (data, query)) = kivact::test::render_with_handle(crate::options(), || {
        let items = scene();
        let data = signal(items);
        let query = signal(String::new());
        (
            (data, query),
            app_outliner(
                data,
                ui::OutlinerState {
                    expanded: (signal(vec![1])).into(),
                    ..ui::OutlinerState::new(ids(data))
                },
            )
            .filter(query),
        )
    });
    assert!(screen.query_by_role(Role::TreeItem, "Cube").is_none());
    query.set("CUBE".into());
    screen.run_until_idle();
    for shown in ["Scene", "Group", "Cube"] {
        assert!(item(&screen, shown).exists(), "{shown}");
    }
    assert!(screen.query_by_role(Role::TreeItem, "Other").is_none());
    assert!(screen.query_by_role(Role::TreeItem, "Lamp").is_none());
    assert_eq!(
        data.get_untracked(),
        scene(),
        "filter preserves application items"
    );
    query.set(String::new());
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::TreeItem, "Cube").is_none());
    assert!(item(&screen, "Other").exists());
    clean(&screen);
}

#[test]
fn rename_is_f2_and_a_double_click_unless_the_item_is_activated() {
    let (screen, (data, activated)) = kivact::test::render_with_handle(crate::options(), || {
        let data = signal(scene());
        let activated = signal(Vec::<u64>::new());
        (
            (data, activated),
            app_outliner(data, ui::OutlinerState::new(ids(data))).on_item_activate(move |id| {
                activated.update(|all| all.push(id));
            }),
        )
    });
    // With an activation callback a double click and Enter activate.
    item(&screen, "Other").double_click();
    assert_eq!(activated.get_untracked(), [4], "once per double click");
    assert!(
        screen
            .query_by_role(Role::TextBox, "Rename tree item")
            .is_none()
    );
    screen.press_key(Key::Enter);
    assert_eq!(activated.get_untracked(), [4, 4]);
    // F2 renames.
    item(&screen, "Lamp").click();
    screen.press_key(Key::F(2));
    screen
        .get_by_role(Role::TextBox, "Rename tree item")
        .type_text("Light");
    screen.press_key(Key::Enter);
    assert_eq!(data.get_untracked()[4].label, "Light");
    assert!(item(&screen, "Light").is_focused());
    clean(&screen);

    // Without one, a double click renames, and Escape keeps the name.
    let (screen, (data, rename)) = kivact::test::render_with_handle(crate::options(), || {
        let data = signal(scene());
        let rename = signal(None::<u64>);
        (
            (data, rename),
            app_outliner(
                data,
                ui::OutlinerState {
                    renaming: rename.into(),
                    ..ui::OutlinerState::new(ids(data))
                },
            ),
        )
    });
    item(&screen, "Cube").double_click();
    assert_eq!(rename.get_untracked(), Some(3));
    screen
        .get_by_role(Role::TextBox, "Rename tree item")
        .type_text("Mesh");
    screen.press_key(Key::Escape);
    assert_eq!(rename.get_untracked(), None);
    assert_eq!(data.get_untracked()[2].label, "Cube");
    assert!(item(&screen, "Cube").is_focused());
    item(&screen, "Cube").press_key(Key::F(2));
    screen
        .get_by_role(Role::TextBox, "Rename tree item")
        .type_text("Mesh");
    screen.press_key(Key::Enter);
    assert_eq!(data.get_untracked()[2].label, "Mesh");
    clean(&screen);
}

#[test]
fn a_drop_is_before_after_or_into_by_the_third_of_the_row_and_reported() {
    let (screen, drops) = kivact::test::render_with_handle(crate::options(), || {
        let (drops, callback) = recorded();
        let data = signal(scene());
        (
            drops,
            app_outliner(data, ui::OutlinerState::new(ids(data)))
                .on_item_drop(move |value| callback.try_call(value)),
        )
    });
    let drop = |id, target, after| ui::TreeDrop { id, target, after };
    drag_over(&screen, "Other", "Group", 0.1);
    drag_over(&screen, "Other", "Lamp", 0.9);
    drag_over(&screen, "Other", "Cube", 0.5);
    drag_over(&screen, "Other", "Scene", 0.5);
    drag_over(&screen, "Cube", "Other", 0.9);
    drag_over(&screen, "Cube", "Scene", 0.1);
    drag_over(&screen, "Lamp", "Cube", 0.1);
    assert_eq!(
        drops.get_untracked(),
        [
            drop(4, Some(1), None),
            drop(4, Some(1), Some(5)),
            drop(4, Some(3), None),
            drop(4, Some(1), Some(5)),
            drop(3, None, Some(4)),
            drop(3, None, None),
            drop(5, Some(2), None),
        ]
    );
    // Never on itself or into its own subtree.
    drag_over(&screen, "Group", "Cube", 0.5);
    drag_over(&screen, "Group", "Group", 0.5);
    assert_eq!(drops.get_untracked().len(), 7);
    clean(&screen);
}

#[test]
fn without_on_item_drop_the_outliner_keeps_the_application_items() {
    let (screen, data) = kivact::test::render_with_handle(crate::options(), || {
        let data = signal(scene());
        (data, ui::outliner(data, ui::OutlinerState::new(ids(data))))
    });
    drag_over(&screen, "Lamp", "Group", 0.1);
    drag_over(&screen, "Other", "Cube", 0.5);
    assert_eq!(data.get_untracked(), scene());
    clean(&screen);
}

#[test]
fn a_cancelled_drag_reports_no_drop() {
    let (screen, drops) = kivact::test::render_with_handle(crate::options(), || {
        let (drops, callback) = recorded();
        let data = signal(scene());
        (
            drops,
            app_outliner(data, ui::OutlinerState::new(ids(data)))
                .on_item_drop(move |value| callback.try_call(value)),
        )
    });
    let from = item(&screen, "Other").rect().expect("other");
    let over = item(&screen, "Group").rect().expect("group");
    screen.mouse_move(from.center().x, from.center().y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(over.center().x, over.center().y - 4.0);
    screen.mouse_move(over.center().x, over.center().y);
    screen.press_key(Key::Escape);
    screen.mouse_up(MouseButton::Primary);
    assert!(drops.get_untracked().is_empty());
    clean(&screen);
}

#[test]
fn the_active_item_is_marked_and_revealed_through_its_collapsed_ancestors() {
    let (screen, (data, active)) = kivact::test::render_with_handle(crate::options(), || {
        let items = scene();
        let data = signal(items);
        let active = signal(None::<u64>);
        (
            (data, active),
            app_outliner(
                data,
                ui::OutlinerState {
                    expanded: (signal(Vec::new())).into(),
                    ..ui::OutlinerState::new(ids(data))
                },
            )
            .active(active),
        )
    });
    assert!(screen.query_by_role(Role::TreeItem, "Cube").is_none());
    active.set(Some(3));
    screen.run_until_idle();
    assert_eq!(data.get_untracked(), scene());
    assert_eq!(item(&screen, "Scene").is_expanded(), Some(true));
    assert_eq!(item(&screen, "Group").is_expanded(), Some(true));
    assert_eq!(item(&screen, "Cube").is_checked(), Some(true));
    assert_eq!(item(&screen, "Scene").is_checked(), Some(false));
    clean(&screen);

    // Off: nothing opens.
    let (screen, (data, active)) = kivact::test::render_with_handle(crate::options(), || {
        let items = scene();
        let data = signal(items);
        let active = signal(None::<u64>);
        (
            (data, active),
            app_outliner(
                data,
                ui::OutlinerState {
                    expanded: (signal(Vec::new())).into(),
                    ..ui::OutlinerState::new(ids(data))
                },
            )
            .active(active)
            .reveal_active(false),
        )
    });
    active.set(Some(3));
    screen.run_until_idle();
    assert_eq!(data.get_untracked(), scene());
    assert_eq!(item(&screen, "Scene").is_expanded(), Some(false));
    clean(&screen);
}

#[test]
fn revealing_the_active_item_scrolls_its_row_into_view() {
    let (screen, active) = kivact::test::render_with_handle(crate::options(), || {
        let items = signal(
            (1..=40)
                .map(|id| ui::TreeItem::new(id, None, format!("Item {id}"), "box"))
                .collect::<Vec<_>>(),
        );
        let active = signal(None::<u64>);
        (
            active,
            scroll(
                app_outliner(items, ui::OutlinerState::new(ids(items)))
                    .active(active)
                    .style(style().shrink(0)),
            )
            .test_id("viewport")
            .style(style().height(120)),
        )
    });
    let view = screen.get_by_test_id("viewport").rect().expect("viewport");
    let inside = |name: &str| {
        let r = item(&screen, name).rect().expect("row");
        r.y >= view.y - 0.5 && r.y + r.height <= view.y + view.height + 0.5
    };
    assert!(!inside("Item 40"));
    active.set(Some(40));
    screen.advance(Duration::from_secs(2));
    assert!(inside("Item 40"), "scrolled to the active row");
    active.set(Some(1));
    screen.advance(Duration::from_secs(2));
    assert!(inside("Item 1"));
    clean(&screen);
}

#[test]
fn a_fixed_cell_takes_its_own_presses_and_clips_when_narrow() {
    let (screen, (selected, locks)) = kivact::test::render_with_handle(crate::options(), || {
        let selected = signal(Vec::<u64>::new());
        let locks = signal(Vec::<u64>::new());
        let items = signal(vec![
            ui::TreeItem::new(1, None, "A long label of the first item", "box"),
            ui::TreeItem::new(2, None, "Second", "box"),
        ]);
        (
            (selected, locks),
            app_outliner(
                items,
                ui::OutlinerState {
                    selection: selected.into(),
                    ..ui::OutlinerState::new(ids(items))
                },
            )
            .restrictions(false)
            .cell(100.0, move |item| {
                let id = item.get_untracked().id;
                row((
                    kivact::ui::button(format!("Lock {id}"))
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .hide_label(true)
                        .icon(kivact::control::icon("click"))
                        .on_press(move || locks.update(|all| all.push(id))),
                    text("a trailing cell that is much too long to fit")
                        .style(style().single_line()),
                ))
            })
            .style(style().width(170)),
        )
    });
    screen.get_by_role(Role::Button, "Lock 2").click();
    assert_eq!(locks.get_untracked(), [2]);
    assert!(
        selected.get_untracked().is_empty(),
        "a press in the cell does not select the row"
    );
    // The icon of the row, left of the cell.
    let second = item(&screen, "Second").rect().expect("row");
    screen.click_at(second.x + 38.0, second.y + second.height / 2.0);
    assert_eq!(selected.get_untracked(), [2]);
    let row_rect = item(&screen, "A long label of the first item")
        .rect()
        .expect("row");
    let cell = screen
        .get_by_role(Role::Button, "Lock 1")
        .rect()
        .expect("button");
    assert!(
        cell.x + cell.width <= row_rect.x + row_rect.width + 0.5,
        "the cell stays inside the row"
    );
    let text_rect = screen.all_by_text("a trailing cell that is much too long to fit")[0]
        .rect()
        .expect("text");
    assert!(
        text_rect.x + text_rect.width <= row_rect.x + row_rect.width + 0.5,
        "the text is clipped to the row"
    );
    clean(&screen);
}

#[test]
fn fixed_cells_keep_width_order_and_reactivity_and_own_their_input() {
    let (screen, (data, selected, rename, presses, toggled, drops, width)) =
        kivact::test::render_with_handle(crate::options(), || {
            let data = signal(vec![
                ui::TreeItem::new(1, None, "Primary", "box"),
                ui::TreeItem::new(2, None, "Second", "box"),
            ]);
            let selected = signal(vec![2]);
            let rename = signal(None::<u64>);
            let presses = signal(0);
            let toggled = signal(false);
            let (drops, on_drop) = recorded();
            let width = signal(280.0);
            (
                (data, selected, rename, presses, toggled, drops, width),
                app_outliner(
                    data,
                    ui::OutlinerState {
                        selection: selected.into(),
                        renaming: rename.into(),
                        ..ui::OutlinerState::new(ids(data))
                    },
                )
                .on_item_drop(move |value| on_drop.try_call(value))
                .restrictions(false)
                .row_height(64.0)
                .cell(80.0, move |item| {
                    let id = item.get_untracked().id;
                    column((
                        kivact::ui::button(format!("Do {id}"))
                            .on_press(move || presses.update(|n| *n += 1)),
                        text(move || format!("cell {}", item.get().label))
                            .style(style().single_line()),
                    ))
                    .role(Role::Group)
                    .name(format!("Action cell {id}"))
                    .style(style().width(kivact::style::pct(100)))
                })
                .cell(80.0, move |item| {
                    let id = item.get_untracked().id;
                    row(kivact::ui::toggle(format!("Flag {id}"), toggled))
                        .role(Role::Group)
                        .name(format!("Flag cell {id}"))
                        .style(style().width(kivact::style::pct(100)))
                })
                .cell(f32::NAN, |_| text("Hidden invalid cell"))
                .style(move || style().width(width.get())),
            )
        });
    let bounds = || {
        let first = screen
            .get_by_role(Role::Group, "Action cell 1")
            .rect()
            .expect("first cell");
        let second = screen
            .get_by_role(Role::Group, "Flag cell 1")
            .rect()
            .expect("second cell");
        assert_eq!(first.width, 80.0);
        assert_eq!(second.width, 80.0);
        assert!(
            first.x + first.width <= second.x,
            "cells follow builder order"
        );
    };
    bounds();
    width.set(240.0);
    screen.run_until_idle();
    bounds();
    assert!(screen.query_by_text("Hidden invalid cell").is_none());
    let button = screen.get_by_role(Role::Button, "Do 1");
    button.click();
    button.press_key(Key::Space);
    button.press_key(Key::Enter);
    assert_eq!(presses.get_untracked(), 3);
    button.press_key(Key::ArrowDown);
    button.press_key(Key::F(2));
    let toggle = screen.get_by_role(Role::Button, "Flag 1");
    toggle.click();
    assert!(toggled.get_untracked());
    toggle.press_key(Key::Space);
    assert!(!toggled.get_untracked());
    toggle.press_key(Key::ArrowRight);
    assert_eq!(selected.get_untracked(), [2]);
    assert_eq!(rename.get_untracked(), None);
    let bounds = button.rect().expect("cell button");
    let start = Point::new(bounds.width / 2.0, bounds.height / 2.0);
    let destination = item(&screen, "Second").rect().expect("second row").center();
    button.drag(
        start,
        Point::new(destination.x - bounds.x, destination.y - bounds.y),
    );
    assert!(drops.get_untracked().is_empty());
    assert_eq!(selected.get_untracked(), [2]);
    data.update(|items| items[0].label = "Updated".to_owned());
    screen.run_until_idle();
    screen.get_by_text("cell Updated");
    clean(&screen);
}

#[test]
fn placeholder_and_label_tone_change_paint_without_disabling_the_item() {
    let (screen, (data, selected, activations)) =
        kivact::test::render_with_handle(crate::options(), || {
            let original = ui::TreeItem::new(1, None, "Unresolved asset", "box");
            assert_eq!(original.tone, ui::LabelTone::Normal);
            assert!(!original.placeholder);
            let data = signal(vec![original]);
            let selected = signal(Vec::<u64>::new());
            let activations = signal(Vec::<u64>::new());
            (
                (data, selected, activations),
                app_outliner(
                    data,
                    ui::OutlinerState {
                        selection: selected.into(),
                        ..ui::OutlinerState::new(ids(data))
                    },
                )
                .restrictions(false)
                .on_item_activate(move |id| activations.update(|all| all.push(id))),
            )
        });
    let original = screen.screenshot().hash();
    data.update(|items| {
        items[0] = items[0]
            .clone()
            .placeholder(true)
            .tone(ui::LabelTone::Error)
    });
    screen.run_until_idle();
    assert_ne!(screen.screenshot().hash(), original);
    let row = item(&screen, "Unresolved asset");
    assert!(!row.is_disabled());
    row.click();
    row.press_key(Key::Enter);
    assert_eq!(selected.get_untracked(), [1]);
    assert_eq!(activations.get_untracked(), [1]);
    assert!(data.get_untracked()[0].placeholder);
    assert_eq!(data.get_untracked()[0].tone, ui::LabelTone::Error);
    clean(&screen);
}

#[test]
fn a_row_height_sets_the_rows_and_a_non_selectable_item_is_disabled() {
    let (screen, selected) = kivact::test::render_with_handle(crate::options(), || {
        let selected = signal(Vec::<u64>::new());
        let mut items = scene();
        items[3].selectable = false;
        let items = signal(items);
        (
            selected,
            app_outliner(
                items,
                ui::OutlinerState {
                    selection: selected.into(),
                    ..ui::OutlinerState::new(ids(items))
                },
            )
            .row_height(40.0),
        )
    });
    let rect = item(&screen, "Scene").rect().expect("row");
    assert_eq!(rect.height, 40.0);
    assert!(item(&screen, "Other").is_disabled());
    screen.get_by_role(Role::TreeItem, "Lamp").click();
    screen.press_key(Key::End);
    assert_eq!(selected.get_untracked(), [5], "the keys skip it");
    clean(&screen);
}

/// Every item's ID, which an outliner state opens.
fn ids(items: Signal<Vec<ui::TreeItem>>) -> Vec<u64> {
    items.with_untracked(|items| items.iter().map(|item| item.id).collect())
}

fn app_outliner(items: Signal<Vec<ui::TreeItem>>, state: ui::OutlinerState) -> ui::Outliner {
    let initial = signal(Vec::new());
    ui::outliner(items, state)
        .on_item_drop(move |drop: ui::TreeDrop| {
            items.update(|items| {
                let Some(from) = items.iter().position(|item| item.id == drop.id) else {
                    return;
                };
                let mut item = items.remove(from);
                item.parent = drop.target;
                let at = match drop.after {
                    Some(after) => items
                        .iter()
                        .position(|item| item.id == after)
                        .map_or(items.len(), |at| at + 1),
                    None => items
                        .iter()
                        .position(|item| item.parent == drop.target)
                        .unwrap_or(from.min(items.len())),
                };
                items.insert(at, item);
            });
        })
        .on_rename(move |(id, name): (u64, String)| {
            if name.trim().is_empty() {
                return;
            }
            items.update(|items| {
                if let Some(item) = items.iter_mut().find(|item| item.id == id) {
                    item.label = name;
                }
            });
        })
        .on_restriction(move |event: ui::EditorEvent<ui::Restriction>| match event {
            ui::EditorEvent::Begin => initial.set(items.get_untracked()),
            ui::EditorEvent::Request(request) => items.update(|items| {
                for item in items
                    .iter_mut()
                    .filter(|item| request.ids.contains(&item.id))
                {
                    match request.flag {
                        ui::RestrictionFlag::Visible => item.visible = request.value,
                        ui::RestrictionFlag::Selectable => item.selectable = request.value,
                        ui::RestrictionFlag::Rendered => item.rendered = request.value,
                    }
                }
            }),
            ui::EditorEvent::Cancel => items.set(initial.get_untracked()),
            ui::EditorEvent::Commit => {}
        })
}

#[test]
fn restriction_requests_are_absolute_and_application_rejection_is_visible() {
    let (screen, (data, events)) = kivact::test::render_with_handle(crate::options(), || {
        let data = signal(scene());
        let events = signal(Vec::new());
        (
            (data, events),
            ui::outliner(
                data,
                ui::OutlinerState {
                    selection: (signal(vec![2, 3])).into(),
                    ..ui::OutlinerState::new(ids(data))
                },
            )
            .on_restriction(move |event| events.update(|events| events.push(event))),
        )
    });
    let button = item(&screen, "Group").get_by_role(Role::Button, "Tree visibility");
    button.a11y(A11yAction::Toggle);
    assert_eq!(
        events.get_untracked(),
        vec![
            ui::EditorEvent::Begin,
            ui::EditorEvent::Request(ui::Restriction {
                ids: vec![2, 3],
                flag: ui::RestrictionFlag::Visible,
                value: false
            }),
            ui::EditorEvent::Commit,
        ]
    );
    assert_eq!(button.is_checked(), Some(true));
    assert_eq!(data.get_untracked(), scene());
    item(&screen, "Group").click();
    screen.press_key(Key::Char('h'));
    assert_eq!(events.get_untracked().len(), 6);
    assert_eq!(data.get_untracked(), scene());
    clean(&screen);
}

#[test]
fn restriction_paint_accumulates_ids_and_cancel_restores_application_items() {
    let (screen, (data, events)) = kivact::test::render_with_handle(crate::options(), || {
        let data = signal(scene());
        let initial = signal(Vec::new());
        let events = signal(Vec::new());
        (
            (data, events),
            app_outliner(data, ui::OutlinerState::new(ids(data))).on_restriction(
                move |event: ui::EditorEvent<ui::Restriction>| {
                    events.update(|events| events.push(event.clone()));
                    match event {
                        ui::EditorEvent::Begin => initial.set(data.get_untracked()),
                        ui::EditorEvent::Request(request) => data.update(|items| {
                            for item in items
                                .iter_mut()
                                .filter(|item| request.ids.contains(&item.id))
                            {
                                item.visible = request.value;
                            }
                        }),
                        ui::EditorEvent::Cancel => data.set(initial.get_untracked()),
                        ui::EditorEvent::Commit => {}
                    }
                },
            ),
        )
    });
    item(&screen, "Group").click();
    let a = item(&screen, "Group")
        .get_by_role(Role::Button, "Tree visibility")
        .rect()
        .unwrap();
    let b = item(&screen, "Cube")
        .get_by_role(Role::Button, "Tree visibility")
        .rect()
        .unwrap();
    screen.mouse_move(a.center().x, a.center().y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(b.center().x, b.center().y);
    assert!(!data.get_untracked()[1].visible && !data.get_untracked()[2].visible);
    assert_eq!(
        events.get_untracked(),
        vec![
            ui::EditorEvent::Begin,
            ui::EditorEvent::Request(ui::Restriction {
                ids: vec![2],
                flag: ui::RestrictionFlag::Visible,
                value: false
            }),
            ui::EditorEvent::Request(ui::Restriction {
                ids: vec![2, 3],
                flag: ui::RestrictionFlag::Visible,
                value: false
            }),
        ]
    );
    screen.press_key(Key::Escape);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        events.get_untracked().last(),
        Some(&ui::EditorEvent::Cancel)
    );
    assert_eq!(data.get_untracked(), scene());
    assert_eq!(
        item(&screen, "Group")
            .get_by_role(Role::Button, "Tree visibility")
            .is_checked(),
        Some(true)
    );
    clean(&screen);
}

#[test]
fn missing_handlers_and_rejected_names_leave_items_unchanged() {
    let (screen, data) = kivact::test::render_with_handle(crate::options(), || {
        let data = signal(scene());
        (data, ui::outliner(data, ui::OutlinerState::new(ids(data))))
    });
    assert!(
        item(&screen, "Group")
            .get_by_role(Role::Button, "Tree visibility")
            .is_disabled()
    );
    item(&screen, "Group").click();
    screen.press_key(Key::Char('h'));
    screen.press_key(Key::F(2));
    screen
        .get_by_role(Role::TextBox, "Rename tree item")
        .type_text("Ignored");
    screen.press_key(Key::Enter);
    assert_eq!(data.get_untracked(), scene());
    clean(&screen);
}

#[test]
fn the_application_can_reject_a_submitted_name() {
    let (screen, (data, names)) = kivact::test::render_with_handle(crate::options(), || {
        let data = signal(scene());
        let names = signal(Vec::<(u64, String)>::new());
        (
            (data, names),
            ui::outliner(data, ui::OutlinerState::new(ids(data)))
                .on_rename(move |request| names.update(|names| names.push(request))),
        )
    });
    item(&screen, "Group").click();
    screen.press_key(Key::F(2));
    screen
        .get_by_role(Role::TextBox, "Rename tree item")
        .type_text("Rejected");
    screen.press_key(Key::Enter);
    assert_eq!(names.get_untracked(), [(2, "Rejected".to_owned())]);
    assert_eq!(data.get_untracked(), scene());
    assert!(item(&screen, "Group").is_focused());
    clean(&screen);
}

#[test]
fn expansion_is_an_application_bound_view_state_separate_from_items() {
    let (screen, (items, expanded)) = kivact::test::render_with_handle(crate::options(), || {
        let items = signal(scene());
        let expanded = signal(vec![1]);
        (
            (items, expanded),
            ui::outliner(
                items,
                ui::OutlinerState {
                    expanded: expanded.into(),
                    ..ui::OutlinerState::new(ids(items))
                },
            ),
        )
    });
    assert!(screen.query_by_role(Role::TreeItem, "Cube").is_none());
    expanded.set(vec![1, 2]);
    screen.run_until_idle();
    assert!(item(&screen, "Cube").exists());
    item(&screen, "Group")
        .get_by_role(Role::Button, "Expand tree item")
        .a11y(A11yAction::Toggle);
    assert!(!expanded.get_untracked().contains(&2));
    assert_eq!(items.get_untracked(), scene());
    clean(&screen);
}

#[test]
fn selectable_and_rendered_restriction_clicks_apply_application_requests() {
    let (screen, items) = kivact::test::render_with_handle(crate::options(), || {
        let items = signal(scene());
        (
            items,
            app_outliner(items, ui::OutlinerState::new(ids(items))),
        )
    });
    item(&screen, "Group")
        .get_by_role(Role::Button, "Tree render visibility")
        .click();
    assert!(!items.get_untracked()[1].rendered);
    item(&screen, "Group")
        .get_by_role(Role::Button, "Tree selection restriction")
        .click();
    assert!(!items.get_untracked()[1].selectable);
    assert!(item(&screen, "Group").is_disabled());
    item(&screen, "Group")
        .get_by_role(Role::Button, "Tree selection restriction")
        .click();
    assert!(items.get_untracked()[1].selectable);
    assert!(!item(&screen, "Group").is_disabled());
    clean(&screen);
}

#[test]
fn single_restriction_request_captures_selection_before_begin_callback() {
    let (screen, (data, selected, events)) =
        kivact::test::render_with_handle(crate::options(), || {
            let data = signal(scene());
            let selected = signal(vec![2, 3]);
            let events = signal(Vec::new());
            (
                (data, selected, events),
                ui::outliner(
                    data,
                    ui::OutlinerState {
                        selection: selected.into(),
                        ..ui::OutlinerState::new(ids(data))
                    },
                )
                .on_restriction(move |event| {
                    if event == ui::EditorEvent::Begin {
                        selected.set(vec![2, 4]);
                        data.update(|items| items[1].visible = false);
                    }
                    events.update(|events| events.push(event));
                }),
            )
        });
    item(&screen, "Group")
        .get_by_role(Role::Button, "Tree visibility")
        .a11y(A11yAction::Toggle);
    assert_eq!(
        events.get_untracked(),
        vec![
            ui::EditorEvent::Begin,
            ui::EditorEvent::Request(ui::Restriction {
                ids: vec![2, 3],
                flag: ui::RestrictionFlag::Visible,
                value: false,
            }),
            ui::EditorEvent::Commit,
        ]
    );
    assert_eq!(selected.get_untracked(), [2, 4]);
    let mut expected = scene();
    expected[1].visible = false;
    assert_eq!(data.get_untracked(), expected);
    clean(&screen);
}
