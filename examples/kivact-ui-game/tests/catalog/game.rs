//! Role: The game's visible source states, exercised through the public consumer driver.
use std::time::Duration;

use kivact::draw::Point;
use kivact::test::{Element, Screen};
use kivact::view::{A11yAction, Key, Modifiers, Role, SortDirection};

use super::{evidence, mount};

fn chapter(screen: &Screen, name: &str) {
    screen.get_by_test_id(&format!("game-nav-{name}")).click();
    screen.advance(Duration::from_millis(400));
    assert!(
        screen
            .get_by_test_id(&format!("game-nav-{name}"))
            .is_selected()
    );
    screen.assert_layout_with(&crate::layout_check());
}

fn clean(screen: &Screen) {
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn fill(input: Element<'_>, value: &str) {
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
    let screen = mount();
    chapter(&screen, "inputs");
    let input = screen.get_by_test_id("game-title-input");
    fill(input, "第 4 区画 点検");
    assert_eq!(input.value().as_deref(), Some("第 4 区画 点検"));
    let code = screen.get_by_role(Role::TextBox, "管理コード");
    fill(code, "OPS-2041");
    assert!(!code.invalid());
    code.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    );
    code.press_key(Key::Backspace);
    assert!(code.invalid());
    evidence(&screen, "game-input-focus");
    chapter(&screen, "selection");
    let all = screen.get_by_role(Role::CheckBox, "全権限");
    assert!(all.is_indeterminate());
    all.click();
    assert_eq!(all.is_checked(), Some(true));
    assert!(!all.is_indeterminate());
    screen.get_by_role(Role::CheckBox, "承認").click();
    assert!(all.is_indeterminate());
    let track = screen.get_by_test_id("game-cpu-slider");
    let rect = track.rect().expect("visible slider bounds");
    track.drag(
        Point::new(rect.width * 0.35, rect.height / 2.),
        Point::new(rect.width * 0.8, rect.height / 2.),
    );
    let slider = screen.get_by_role(Role::Slider, "CPU 上限");
    let value = slider.value_range().2.expect("slider exposes its value");
    assert!((70. ..=90.).contains(&value), "dragged CPU value {value}");
    slider.a11y_with_value(A11yAction::SetValue, "37.5");
    assert_eq!(slider.value_range().2, Some(37.5));
    evidence(&screen, "game-selection-mixed-slider");
    clean(&screen);
}

#[test]
fn game_table_sort_filter_bulk_and_drawer_preserve_record_keys() {
    let screen = mount();
    chapter(&screen, "table");
    let page_size = screen.get_by_test_id("game-table-page-size");
    page_size.press_key(Key::ArrowDown);
    assert_eq!(page_size.value().as_deref(), Some("50"));
    let all = screen.get_by_role(Role::CheckBox, "表示行をすべて選択");
    screen
        .get_by_role(Role::CheckBox, "OPS-2041 を選択")
        .click();
    assert!(all.is_indeterminate());
    assert!(screen.get_by_role(Role::Row, "OPS-2041").is_selected());
    evidence(&screen, "game-table-partial-bulk");
    let progress = screen.get_by_role(Role::ColumnHeader, "進捗");
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
    fill(screen.get_by_test_id("game-table-search"), "OPS-2041");
    assert!(screen.query_by_role(Role::Row, "OPS-2036").is_none());
    assert!(screen.get_by_role(Role::Row, "OPS-2041").is_selected());
    screen.get_by_test_id("game-bulk-approve").click();
    let row = screen.get_by_role(Role::Row, "OPS-2041");
    row.get_by_text("PASSED");
    row.get_by_text("100%");
    evidence(&screen, "game-table-filter-approved");
    screen.mouse_leave();
    let details = screen.get_by_role(Role::Button, "OPS-2041 の詳細");
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
    details.click();
    assert!(screen.get_by_role(Role::Dialog, "案件詳細").exists());
    evidence(&screen, "game-table-row-drawer");
    screen.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::Dialog, "案件詳細").is_none());
    clean(&screen);
}
