//! Role: External-consumer tests: no internal Kivact or renderer imports are available.

use kivact::draw::Point;
use kivact::prelude::*;
use kivact::test::RenderOptions;
use kivact::ui::{Command, MenuEntry, toaster};
use kivact::view::{A11yAction, LiveRegion};
use kivact_ui_pop as ui;
use std::time::Duration;

#[path = "consumer/dates.rs"]
mod dates;
#[path = "consumer/interactions.rs"]
mod interactions;

#[test]
fn button_states_and_user_modifiers_use_public_contract() {
    let (screen, count) = kivact::test::render_with_handle(crate::options(), || {
        let count = signal(0);
        (
            count,
            column((
                kivact::ui::button("Save")
                    .on_press(move || count.update(|v| *v += 1))
                    .style(style().height(40))
                    .test_id("save"),
                kivact::ui::button("Disabled")
                    .disabled(true)
                    .on_press(move || count.update(|v| *v += 100)),
            )),
        )
    });
    let button = screen.get_by_role(Role::Button, "Save");
    button.click();
    button.press_key(Key::Enter);
    assert_eq!(count.get_untracked(), 2);
    assert_eq!(button.rect().map(|r| r.height), Some(40.0));
    assert!(screen.get_by_role(Role::Button, "Disabled").is_disabled());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn checkbox_switch_and_mixed_states_are_accessible() {
    let (screen, (checked, on)) = kivact::test::render_with_handle(crate::options(), || {
        let checked = signal(false);
        let on = signal(true);
        (
            (checked, on),
            column((
                kivact::ui::checkbox("Reports", checked),
                kivact::ui::checkbox("Partial", kivact::signal(false)).indeterminate(true),
                kivact::ui::switch("Sync", on),
            )),
        )
    });
    let check = screen.get_by_role(Role::CheckBox, "Reports");
    check.press_key(Key::Space);
    assert!(checked.get_untracked());
    assert!(
        screen
            .get_by_role(Role::CheckBox, "Partial")
            .is_indeterminate()
    );
    screen.get_by_role(Role::Switch, "Sync").click();
    assert!(!on.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn toggles_and_accordion_follow_the_shared_toggle_rules() {
    let (screen, (check, switch, bold, open)) =
        kivact::test::render_with_handle(crate::options(), || {
            let check = signal(false);
            let switch = signal(false);
            let bold = signal(false);
            let open = signal(false);
            (
                (check, switch, bold, open),
                column((
                    kivact::ui::checkbox("Reports", check),
                    kivact::ui::switch("Sync", switch),
                    kivact::ui::toggle("Bold", bold),
                    kivact::ui::disclosure("Details", open, || text("Body")),
                    kivact::ui::checkbox("Locked", kivact::signal(false)).disabled(true),
                )),
            )
        });
    let check_box = screen.get_by_role(Role::CheckBox, "Reports");
    let switch_node = screen.get_by_role(Role::Switch, "Sync");
    let button = screen.get_by_role(Role::Button, "Bold");
    check_box.press_key(Key::Enter);
    switch_node.press_key(Key::Enter);
    assert!(!check.get_untracked() && !switch.get_untracked());
    button.press_key(Key::Enter);
    assert!(bold.get_untracked(), "Enter flips a toggle button");
    assert_eq!(button.is_checked(), Some(true));
    for node in [&check_box, &switch_node, &button] {
        node.press_key(Key::Space);
        assert_eq!(node.value(), None, "role and checked, no On/Off text");
    }
    assert!(check.get_untracked() && switch.get_untracked() && !bold.get_untracked());
    check_box.a11y(A11yAction::Toggle);
    switch_node.a11y(A11yAction::Activate);
    assert!(!check.get_untracked() && !switch.get_untracked());
    let locked = screen.get_by_role(Role::CheckBox, "Locked");
    assert!(locked.is_disabled());
    locked.force_click();
    locked.a11y(A11yAction::Toggle);
    assert_eq!(locked.is_checked(), Some(false));

    let details = screen.get_by_role(Role::Button, "Details");
    assert_eq!(details.is_expanded(), Some(false));
    details.press_key(Key::Enter);
    assert!(open.get_untracked());
    details.press_key(Key::Space);
    assert!(!open.get_untracked());
    details.a11y(A11yAction::Toggle);
    assert_eq!(details.is_expanded(), Some(true));
    details.a11y(A11yAction::Collapse);
    assert!(!open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn slider_drag_keyboard_and_accessibility_share_one_binding() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(30.0f64);
        (
            value,
            kivact::ui::slider("Volume", value)
                .range(
                    kivact::ui::ScalarRange::new(0.0, 100.0)
                        .and_then(|range| range.with_step(1.0))
                        .expect("finite ordered range"),
                )
                .test_id("volume")
                .style(style().width(300)),
        )
    });
    let slider = screen.get_by_role(Role::Slider, "Volume");
    slider.press_key(Key::ArrowRight);
    assert_eq!(value.get_untracked(), 31.0);
    slider.a11y(A11yAction::Increment);
    assert_eq!(value.get_untracked(), 32.0);
    slider.a11y_with_value(A11yAction::SetValue, "37.5");
    assert_eq!(value.get_untracked(), 38.0);
    slider.a11y_with_value(A11yAction::SetValue, "NaN");
    assert_eq!(value.get_untracked(), 38.0);
    slider.a11y_with_value(A11yAction::SetValue, "-1000");
    assert_eq!(value.get_untracked(), 0.0);
    screen
        .get_by_test_id("volume")
        .drag(Point::new(40.0, 10.0), Point::new(220.0, 10.0));
    assert!(value.get_untracked() > 60.0);
    assert_eq!(slider.value_range().2, Some(value.get_untracked()));
    slider.press_key(Key::End);
    assert_eq!(value.get_untracked(), 100.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn inputs_and_tag_editor_commit_native_edits() {
    let (screen, (draft, tags, memo)) = kivact::test::render_with_handle(crate::options(), || {
        let draft = signal(String::new());
        let tags = signal(vec!["one".into()]);
        let memo = signal(String::new());
        (
            (draft, tags, memo),
            column((
                kivact::ui::text_input("Name", draft).placeholder("Enter name"),
                ui::resizable_text_area(kivact::ui::text_area("Resizable note", memo), 80.0, 72.0),
                kivact::ui::text_area("Fixed note", kivact::signal(String::new())),
                kivact::ui::tag_input(
                    "Add tag",
                    tags,
                    move |tag| tags.update(|items| items.push(tag)),
                    move |index| {
                        tags.update(|items| {
                            items.remove(index);
                        })
                    },
                ),
            ))
            .style(style().width(360).gap(8)),
        )
    });
    screen
        .get_by_role(Role::TextBox, "Name")
        .type_text("Minato");
    assert_eq!(draft.get_untracked(), "Minato");
    let note = screen.get_by_role(Role::TextBox, "Resizable note");
    note.type_text("Keep this draft");
    let initial = note.rect().expect("note laid out");
    assert_eq!(initial.height, 80.0);
    let resize = |delta: f32| {
        let grip = screen.get_by_role(Role::Splitter, "Resize text area");
        grip.hover();
        let rect = grip.rect().expect("resize grip laid out");
        let x = rect.x + rect.width / 2.0;
        let y = rect.y + rect.height / 2.0;
        screen.mouse_move(x, y);
        screen.mouse_down(kivact::view::MouseButton::Primary);
        screen.mouse_move(x, y + delta / 2.0);
        screen.mouse_move(x, y + delta);
        screen.mouse_up(kivact::view::MouseButton::Primary);
    };
    resize(48.0);
    assert_eq!(note.rect().unwrap().height, 128.0);
    assert_eq!(note.rect().unwrap().width, initial.width);
    assert_eq!(memo.get_untracked(), "Keep this draft");
    assert!(note.is_focused());
    screen.type_text(" after resizing");
    assert_eq!(memo.get_untracked(), "Keep this draft after resizing");
    resize(-200.0);
    assert_eq!(note.rect().unwrap().height, 72.0);
    assert_eq!(note.rect().unwrap().width, initial.width);
    assert!(note.is_focused());
    screen.type_text(".");
    assert_eq!(memo.get_untracked(), "Keep this draft after resizing.");
    let grip = screen.get_by_role(Role::Splitter, "Resize text area");
    grip.press_key(Key::ArrowDown);
    assert_eq!(note.rect().unwrap().height, 80.0);
    grip.press_key(Key::ArrowDown);
    assert_eq!(note.rect().unwrap().height, 88.0);
    grip.press_key(Key::ArrowUp);
    assert_eq!(note.rect().unwrap().height, 80.0);
    grip.press_key(Key::Home);
    assert_eq!(note.rect().unwrap().height, 72.0);
    grip.a11y(A11yAction::Increment);
    assert_eq!(note.rect().unwrap().height, 80.0);
    grip.a11y_with_value(A11yAction::SetValue, "-100");
    assert_eq!(note.rect().unwrap().height, 72.0);
    grip.a11y_with_value(A11yAction::SetValue, "NaN");
    assert_eq!(note.rect().unwrap().height, 72.0);
    assert_eq!(
        screen
            .get_by_role(Role::TextBox, "Fixed note")
            .rect()
            .unwrap()
            .height,
        80.0
    );
    let tag = screen.get_by_role(Role::TextBox, "Add tag");
    tag.type_text("two");
    tag.press_key(Key::Enter);
    assert_eq!(tags.get_untracked(), vec!["one", "two"]);
    tag.press_key(Key::Backspace);
    assert_eq!(tags.get_untracked(), vec!["one"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn modal_dismisses_and_restores_focus_to_its_opener() {
    let (screen, open) = kivact::test::render_with_handle(crate::options(), || {
        let open = signal(false);
        (
            open,
            column((
                kivact::ui::button("Open").on_press(move || open.set(true)),
                kivact::ui::dialog("Confirm", open)
                    .content(|| kivact::ui::text_input("Memo", kivact::signal(String::new())))
                    .footer(move || kivact::ui::button("OK").on_press(move || open.set(false))),
            )),
        )
    });
    let opener = screen.get_by_role(Role::Button, "Open");
    opener.click();
    screen.get_by_role(Role::Dialog, "Confirm");
    screen.get_by_role(Role::TextBox, "Memo").type_text("Ready");
    assert_eq!(
        screen.get_by_role(Role::TextBox, "Memo").value(),
        Some("Ready".to_string())
    );
    screen.get_by_role(Role::Button, "OK").click();
    assert!(!open.get_untracked());
    assert!(opener.is_focused());
    opener.click();
    screen.get_by_role(Role::TextBox, "Memo").focus();
    screen.press_key(Key::Escape);
    assert!(!open.get_untracked());
    assert!(screen.query_by_role(Role::Dialog, "Confirm").is_none());
    assert!(opener.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn table_search_sort_selection_paging_are_external_consumer_behavior() {
    #[derive(Clone, PartialEq)]
    struct Member {
        id: u64,
        name: &'static str,
        level: u32,
    }
    let (screen, selected) = kivact::test::render_with_handle(crate::options(), || {
        let selected = signal(Vec::<u64>::new());
        let sort = signal(None);
        let page = signal(0_usize);
        (
            selected,
            kivact::ui::data_table(
                vec![
                    Member {
                        id: 1,
                        name: "Beta",
                        level: 2,
                    },
                    Member {
                        id: 2,
                        name: "Alpha",
                        level: 10,
                    },
                    Member {
                        id: 3,
                        name: "Gamma",
                        level: 4,
                    },
                ],
                |m: &Member| m.id,
                vec![
                    kivact::ui::DataColumn::new("Name", |m: &Member| m.name.to_owned())
                        .width(150.0)
                        .compare(|a, b| a.name.cmp(b.name))
                        .filter(|m, query| m.name.contains(query))
                        .cell(|m| text(m.name)),
                    kivact::ui::DataColumn::new("Level", |m: &Member| m.level.to_string())
                        .width(80.0)
                        .compare(|a, b| a.level.cmp(&b.level))
                        .numeric(true),
                ],
            )
            .selection(selected)
            .sort(sort)
            .filter(signal(String::new()))
            .page(page, 2),
        )
    });
    screen.get_by_role(Role::ColumnHeader, "Name").click();
    screen.get_by_text("Alpha");
    screen.get_by_role(Role::CheckBox, "Select Alpha").click();
    assert_eq!(selected.get_untracked(), vec![2]);
    assert!(
        screen
            .get_by_role(Role::CheckBox, "Select all")
            .is_indeterminate()
    );
    screen.get_by_role(Role::Button, "Next").click();
    screen.get_by_text("Gamma");
    screen
        .get_by_role(Role::TextBox, "Filter…")
        .type_text("Beta");
    assert!(screen.query_by_text("Gamma").is_none());
    screen.get_by_text("Beta");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn notification_has_live_semantics_and_expires_on_the_host_clock() {
    let (screen, toaster) = kivact::test::render_with_handle(crate::options(), || {
        let toaster = toaster();
        toaster.max_visible(4);
        (toaster, kivact::ui::toast_region(toaster))
    });
    toaster.push(
        kivact::ui::Toast::new("Saved")
            .tone(kivact::ui::Tone::Success)
            .body("Request updated"),
    );
    toaster.push(
        kivact::ui::Toast::new("Failed")
            .tone(kivact::ui::Tone::Danger)
            .body("Request rejected"),
    );
    toaster.push(
        kivact::ui::Toast::new("Heads up")
            .tone(kivact::ui::Tone::Warning)
            .body("Quota almost used"),
    );
    screen.run_until_idle();
    assert!(screen.accessibility_tree().contains("Saved"));
    let saved = screen.get_by_role(Role::Status, "Saved");
    let groups = screen.all_by_role(Role::Group, "");
    assert_eq!(groups[0].live_region(), LiveRegion::Polite);
    assert_eq!(groups[1].live_region(), LiveRegion::Assertive);
    assert!(saved.exists());
    screen.get_by_role(Role::Alert, "Failed");
    screen.get_by_role(Role::Alert, "Heads up");
    assert_eq!(screen.all_by_role(Role::Button, "Close").len(), 3);
    screen.advance(Duration::from_millis(4999));
    assert!(screen.query_by_text("Saved").is_some());
    screen.advance(Duration::from_millis(1));
    assert!(screen.query_by_text("Saved").is_none());
    assert!(screen.query_by_text("Failed").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn theme_switch_changes_existing_nodes_and_pixels_without_remounting_state() {
    let (screen, (dark, checked)) = kivact::test::render_with_handle(crate::options(), || {
        let dark = signal(false);
        let checked = signal(true);
        (
            (dark, checked),
            kivact::view::provide_theme(
                move || {
                    ui::theme(if dark.get() {
                        kivact::style::ThemeMode::Dark
                    } else {
                        kivact::style::ThemeMode::Light
                    })
                },
                move || ui::panel(kivact::ui::checkbox("Sync", checked)),
            ),
        )
    });
    let before = screen.screenshot().hash();
    dark.set(true);
    screen.run_until_idle();
    assert!(checked.get_untracked());
    assert_ne!(screen.screenshot().hash(), before);
    assert!(
        screen
            .get_by_role(Role::CheckBox, "Sync")
            .is_checked()
            .unwrap_or(false)
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn command_search_rebuilds_rows_and_executes_the_filtered_action() {
    let (screen, (open, chosen)) = kivact::test::render_with_handle(crate::options(), || {
        let open = signal(true);
        let chosen = signal(0usize);
        (
            (open, chosen),
            kivact::ui::command_palette(
                open,
                vec![
                    Command::new(1, "Alpha").icon("circle-dot"),
                    Command::new(2, "Beta").icon("circle-dot"),
                ],
                move |key: usize| chosen.set(key),
            )
            .footer(|| text("↑↓ Move   Enter Run   Esc Close")),
        )
    });
    screen.get_by_text("↑↓ Move   Enter Run   Esc Close");
    let bounds = screen
        .get_by_role(Role::Dialog, "Command palette")
        .rect()
        .expect("palette bounds");
    assert_eq!(bounds.width, 560.0, "Pop owns the palette panel width");
    let search = screen.get_by_role(Role::TextBox, "Search commands");
    search.type_text("Beta");
    assert!(screen.query_by_role(Role::Option, "Alpha").is_none());
    screen.get_by_role(Role::Option, "Beta");
    search.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), 2);
    assert!(!open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn menu_skips_disabled_items_and_restores_the_custom_trigger() {
    let (screen, chosen) = kivact::test::render_with_handle(crate::options(), || {
        let chosen = signal(0usize);
        (
            chosen,
            kivact::ui::menu(
                "Actions",
                vec![
                    MenuEntry::item(100, "Unavailable").disabled(true),
                    MenuEntry::item(1, "First"),
                    MenuEntry::item(101, "Blocked").disabled(true),
                    MenuEntry::item(2, "Last"),
                ],
                move |key: usize| chosen.set(key),
            )
            .attach(
                kivact::ui::button("Actions")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .hide_label(true)
                    .icon(kivact::control::icon("dots")),
            ),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "Actions");
    trigger.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::MenuItem, "First").is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::MenuItem, "Last").is_focused());
    screen.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), 2);
    assert!(trigger.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn dropzone_reports_public_file_events_and_platform_picker_results() {
    let (screen, paths) = kivact::test::render_with_handle(crate::options(), || {
        let paths = signal(Vec::<std::path::PathBuf>::new());
        (
            paths,
            ui::dropzone("Upload")
                .hint("Select reports")
                .filters(vec![("Reports".into(), vec!["pdf".into()])])
                .on_files(move |files| paths.set(files)),
        )
    });
    screen.get_by_role(Role::Text, "Select reports");
    let dropzone = screen.get_by_role(Role::Button, "Upload");
    let rect = dropzone
        .rect()
        .expect("invariant: drop target was laid out");
    let point = Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
    screen.file_drag(
        kivact::view::FilePhase::Enter,
        vec!["report.pdf".into()],
        point,
    );
    screen.file_drag(
        kivact::view::FilePhase::Drop,
        vec!["report.pdf".into()],
        point,
    );
    assert_eq!(
        paths.get_untracked(),
        vec![std::path::PathBuf::from("report.pdf")]
    );
    screen.respond_files_dialog(Some(vec!["selected.png".into(), "調査.pdf".into()]));
    dropzone.press_key(Key::Enter);
    assert_eq!(
        paths.get_untracked(),
        vec![
            std::path::PathBuf::from("selected.png"),
            std::path::PathBuf::from("調査.pdf")
        ]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn options() -> RenderOptions {
    RenderOptions {
        theme: ui::theme(kivact::style::ThemeMode::Light),
        reduced_motion: true,
        ..RenderOptions::from_env()
    }
}
