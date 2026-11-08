//! Role: All source chapters remain reachable in the complete, font-matched shell.
//! Optional artifacts are produced by the same public test driver with optional window mirroring
//! that consumers use; no library reaches into an implementation crate.
//! The game's visible source states, exercised through the public consumer driver.
#![warn(redundant_imports)]

use kivact::geometry::Point;
use kivact::test::{Element, LayoutCheck, RenderOptions, Screen, TestFonts, render_with};
use kivact::view::{CheckedState, Key, Modifiers, Role, SortDirection};
use kivact_ui_game_catalog::{catalog, fonts, locale, messages};
use std::time::Duration;

#[test]
fn game_source_specimens_scroll_in_the_original_shell() {
    let screen = mount();
    screen.evidence("game-initial");
    chapters(
        &screen,
        &[
            "foundations",
            "buttons",
            "inputs",
            "selection",
            "navigation",
            "display",
            "table",
            "feedback",
            "overlays",
            "layout",
        ],
    );
}

fn mount() -> Screen {
    render_with(
        RenderOptions::with_theme(kivact_ui_game::theme(kivact::style::ThemeMode::Light))
            .window_size(1846., 969.)
            .scale(1.)
            .locale(locale())
            .messages(messages())
            .reduced_motion()
            .fonts(TestFonts::Only(
                fonts().iter().map(|&bytes| bytes.into()).collect(),
            )),
        catalog,
    )
}

/// Preserve the catalog layout exceptions. The big numbers behind the cards bleed out of them
/// on purpose and are ignored by test id.
fn layout_check() -> LayoutCheck {
    ["01", "02", "03", "04"]
        .into_iter()
        .fold(LayoutCheck::default(), |check, number| {
            check.ignore_test_id(format!("deco-number-{number}"))
        })
}

fn chapters(screen: &Screen, chapters: &[&str]) {
    let prefix = "game";
    for (index, chapter) in chapters.iter().enumerate() {
        let nav = screen.get_by_test_id(&format!("{prefix}-nav-{chapter}"));
        nav.click();
        screen.advance(Duration::from_millis(400));
        assert!(nav.is_selected(), "{prefix}/{chapter} remains selected");
        let section_id = format!("game-section-{index}");
        let section = screen
            .get_by_test_id(&section_id)
            .rect()
            .expect("chapter geometry");
        // Selecting the sidebar alone is insufficient: the chapter must actually
        // move into the viewport. The final sections clamp at the scroll limit.
        let final_id = format!("game-section-{}", chapters.len() - 1);
        let end = screen
            .get_by_test_id(&final_id)
            .rect()
            .expect("final chapter geometry");
        let anchor = 72.0;
        let at_scroll_end = section.y > anchor + 8.0 && end.y + end.height <= 971.0;
        if at_scroll_end {
            assert!(
                (0.0..=600.0).contains(&section.y),
                "{prefix}/{chapter} visible at scroll limit: {section:?}"
            );
        } else {
            assert!(
                (section.y - anchor).abs() <= 8.0,
                "{prefix}/{chapter} scrolls to {anchor}: {section:?}"
            );
        }
        assert!(
            screen.errors().is_empty(),
            "{prefix}/{chapter}: {:?}",
            screen.errors()
        );
        screen.assert_layout_with(&layout_check());
        screen.evidence(&format!("{prefix}-{chapter}"));
    }
}

fn chapter(screen: &Screen, name: &str) {
    screen.get_by_test_id(&format!("game-nav-{name}")).click();
    screen.advance(Duration::from_millis(400));
    assert!(
        screen
            .get_by_test_id(&format!("game-nav-{name}"))
            .is_selected()
    );
    screen.assert_layout_with(&layout_check());
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
    chapter(&screen, "selection");
    let all = screen.get_by_role(Role::CheckBox, "全権限");
    assert_eq!(all.checked(), Some(CheckedState::Indeterminate));
    all.click();
    assert_eq!(all.checked(), Some(CheckedState::Checked));
    assert_ne!(all.checked(), Some(CheckedState::Indeterminate));
    screen.get_by_role(Role::CheckBox, "承認").click();
    assert_eq!(all.checked(), Some(CheckedState::Indeterminate));
    let track = screen.get_by_test_id("game-cpu-slider");
    let rect = track.rect().expect("visible slider bounds");
    track.drag(
        Point::new(rect.width * 0.35, rect.height / 2.),
        Point::new(rect.width * 0.8, rect.height / 2.),
    );
    let slider = screen.get_by_role(Role::Slider, "CPU 上限");
    let value = slider.value_range().2.expect("slider exposes its value");
    assert!((70. ..=90.).contains(&value), "dragged CPU value {value}");
    slider.a11y_set_value("37.5");
    assert_eq!(slider.value_range().2, Some(37.5));
    screen.evidence("game-selection-mixed-slider");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
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
    assert_eq!(all.checked(), Some(CheckedState::Indeterminate));
    assert!(screen.get_by_role(Role::Row, "OPS-2041").is_selected());
    screen.evidence("game-table-partial-bulk");
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
    screen.evidence("game-table-filter-approved");
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
    screen.evidence("game-table-row-drawer");
    screen.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::Dialog, "案件詳細").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn game_effects_page_mounts_its_samples_with_main_roles() {
    let screen = mount();
    chapter(&screen, "effects");
    screen.get_by_role(Role::Heading, "EFFECTS");
    screen.get_by_role(Role::Button, "カードを裏返す");
    screen.get_by_role(Role::Button, "パネルを回す");
    screen.get_by_role(Role::Button, "再生し直す");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
