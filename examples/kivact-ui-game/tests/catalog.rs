//! Role: The game catalog's tests: every page opens cleanly.
//! The controls of its pages change real values.

use catalog_shell::check::{
    every_page_character_is_in_the_fonts, every_page_opens_cleanly, mount, open,
};
use kivact::geometry::Point;
use kivact::style::ThemeMode;
use kivact::test::{Locator, Screen};
use kivact::view::{CheckedState, Key, Modifiers, Role, SortDirection};
use kivact_ui_game_catalog::{THEME, pages};

#[test]
fn every_page_opens_cleanly_in_both_modes() {
    every_page_opens_cleanly("Game", pages, THEME, &[]);
}

#[test]
fn every_page_character_is_in_the_catalog_fonts() {
    every_page_character_is_in_the_fonts(pages, THEME);
}

/// The catalog in the light mode, with the page `slug` open.
fn mount_page(slug: &str) -> Screen {
    let screen = mount("Game", pages, THEME, ThemeMode::Light);
    open(&screen, slug);
    screen
}

/// Scroll the page until the node `test_id` is inside the window.
fn scroll_into_page(screen: &Screen, test_id: &str) {
    let page = screen.get_by_test_id("catalog-page");
    let view = page.rect().expect("the page is laid out");
    for _ in 0..8 {
        let item = screen
            .get_by_test_id(test_id)
            .rect()
            .expect("the node is laid out");
        if item.y < view.y {
            page.scroll(0., item.y - view.y - 8.);
        } else if item.y + item.height > view.y + view.height {
            page.scroll(0., item.y + item.height - view.y - view.height + 8.);
        } else {
            return;
        }
    }
    panic!("{test_id} did not scroll into the page");
}

fn select_all_text(input: Locator<'_>, value: &str) {
    input.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    input.type_text(value);
}

#[test]
fn game_input_mixed_selection_and_pointer_slider_change_real_values() {
    let screen = mount_page("inputs");
    let input = screen.get_by_test_id("game-title-input");
    select_all_text(input, "Section 4 inspection");
    assert_eq!(input.value().as_deref(), Some("Section 4 inspection"));
    let code = screen.get_by_role(Role::TextBox, "Access code");
    select_all_text(code, "OPS-2041");
    assert!(!code.is_invalid());
    code.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    code.press_key(Key::Backspace);
    assert!(code.is_invalid());
    screen.evidence("game-input-focus");
    open(&screen, "selection");
    let all = screen.get_by_role(Role::CheckBox, "All permissions");
    assert_eq!(all.checked(), Some(CheckedState::Indeterminate));
    all.click();
    assert_eq!(all.checked(), Some(CheckedState::Checked));
    assert_ne!(all.checked(), Some(CheckedState::Indeterminate));
    screen.get_by_role(Role::CheckBox, "Approve").click();
    assert_eq!(all.checked(), Some(CheckedState::Indeterminate));
    scroll_into_page(&screen, "game-cpu-slider");
    let track = screen.get_by_test_id("game-cpu-slider");
    let rect = track.rect().expect("visible slider bounds");
    track.drag(
        Point::new(rect.width * 0.35, rect.height / 2.),
        Point::new(rect.width * 0.8, rect.height / 2.),
    );
    let slider = screen.get_by_role(Role::Slider, "CPU limit");
    let value = slider.value_range().2.expect("slider exposes its value");
    assert!((70. ..=90.).contains(&value), "dragged CPU value {value}");
    slider.a11y_set_value("37.5");
    assert_eq!(slider.value_range().2, Some(37.5));
    screen.evidence("game-selection-mixed-slider");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn game_table_sort_filter_bulk_and_drawer_preserve_record_keys() {
    let screen = mount_page("data-table");
    let page_size = screen.get_by_test_id("game-table-page-size");
    page_size.press_key(Key::ArrowDown);
    assert_eq!(page_size.value().as_deref(), Some("50"));
    let all = screen.get_by_role(Role::CheckBox, "Select all");
    screen
        .get_by_role(Role::CheckBox, "Select OPS-2041")
        .click();
    assert_eq!(all.checked(), Some(CheckedState::Indeterminate));
    assert!(screen.get_by_role(Role::Row, "OPS-2041").is_selected());
    screen.evidence("game-table-partial-bulk");
    let progress = screen.get_by_role(Role::ColumnHeader, "Progress");
    progress.click();
    assert_eq!(progress.sort(), SortDirection::Ascending);
    let zero = screen
        .get_by_role(Role::Row, "OPS-2036")
        .rect()
        .expect("zero row")
        .y;
    let twenty = screen
        .get_by_role(Role::Row, "OPS-2039")
        .rect()
        .expect("twenty row")
        .y;
    let hundred = screen
        .get_by_role(Role::Row, "OPS-2040")
        .rect()
        .expect("hundred row")
        .y;
    assert!(
        zero < twenty && twenty < hundred,
        "numeric progress ordering"
    );
    select_all_text(screen.get_by_test_id("game-table-search"), "OPS-2041");
    assert!(screen.query_by_role(Role::Row, "OPS-2036").is_none());
    assert!(screen.get_by_role(Role::Row, "OPS-2041").is_selected());
    screen.get_by_test_id("game-bulk-approve").click();
    let row = screen.get_by_role(Role::Row, "OPS-2041");
    row.get_by_text("PASSED");
    row.get_by_text("100%");
    screen.evidence("game-table-filter-approved");
    screen.mouse_leave();
    let details = screen.get_by_role(Role::Button, "Details of OPS-2041");
    reveal_checks(&screen, row, details);
    details.click();
    assert!(
        screen
            .get_by_role(Role::Dialog, "Operation details")
            .exists()
    );
    screen.evidence("game-table-row-drawer");
    screen.press_key(Key::Escape);
    assert!(
        screen
            .query_by_role(Role::Dialog, "Operation details")
            .is_none()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn game_effects_page_mounts_its_samples_with_main_roles() {
    let screen = mount_page("effects");
    screen.get_by_role(Role::Button, "Flip card");
    screen.get_by_role(Role::Button, "Turn panel");
    screen.get_by_role(Role::Button, "Replay");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn window_status_shows_the_last_command_after_run() {
    let screen = mount_page("native-layout");
    let tools = screen.get_by_test_id("game-window-toolbar");
    let status = || screen.get_by_test_id("game-window-status").text();
    tools.get_by_role(Role::Button, "Run").click();
    assert_eq!(status(), "Running");
    tools.get_by_role(Role::Button, "Save").click();
    assert!(status().starts_with("Saved "), "{}", status());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// The row action is hidden until the row is hovered or the action is focused.
fn reveal_checks(screen: &Screen, row: Locator<'_>, details: Locator<'_>) {
    let bounds = details.rect().expect("detail action geometry");
    let crop = || {
        screen
            .screenshot()
            .crop(
                bounds.x as u32,
                bounds.y as u32,
                bounds.width as u32,
                bounds.height as u32,
            )
            .hash()
    };
    let hidden = crop();
    row.hover();
    assert_ne!(hidden, crop(), "the row's hover reveals its action");
    screen.mouse_leave();
    screen.press_key(Key::Tab);
    details.focus();
    assert!(details.is_focused());
    assert_ne!(
        hidden,
        crop(),
        "keyboard focus reveals the action and its focus ring"
    );
}
