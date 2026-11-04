//! Role: All source chapters remain reachable in the complete, font-matched shell.
//! Optional artifacts are produced by the same public test driver with optional window mirroring
//! that consumers use; no library reaches into an implementation crate.
//! The game's visible source states, exercised through the public consumer driver.
#![warn(redundant_imports)]

use kivact::draw::Point;
use kivact::test::{Element, LayoutCheck, RenderOptions, Screen, TestFonts, render_with};
use kivact::view::{CheckedState, Key, Modifiers, Role, SortDirection};
use kivact_ui_game_catalog::{catalog, fonts, locale, messages};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn game_source_specimens_scroll_in_the_original_shell() {
    let screen = mount();
    evidence(&screen, "game-initial");
    chapters(
        &screen,
        "game",
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
        RenderOptions {
            width: 1846.,
            height: 969.,
            scale: 1.,
            locale: Some(locale()),
            messages: messages(),
            reduced_motion: true,
            fonts: TestFonts::Only(fonts().iter().map(|&bytes| Arc::from(bytes)).collect()),
            ..RenderOptions::from_env()
        },
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

fn chapters(screen: &Screen, prefix: &str, chapters: &[&str]) {
    for (index, chapter) in chapters.iter().enumerate() {
        let nav = screen.get_by_test_id(&format!("{prefix}-nav-{chapter}"));
        if prefix == "creative" {
            let scroll = screen.get_by_test_id("creative-nav-scroll");
            let viewport = scroll.rect().expect("sidebar viewport");
            let item = nav.rect().expect("sidebar item");
            if item.y + item.height > viewport.y + viewport.height {
                scroll.scroll(
                    0.,
                    item.y + item.height - viewport.y - viewport.height + 12.,
                );
            }
        }
        nav.click();
        screen.advance(Duration::from_millis(400));
        assert!(nav.is_selected(), "{prefix}/{chapter} remains selected");
        let section_id = match prefix {
            "basic" => format!("basic-section-{chapter}"),
            "game" => format!("game-section-{index}"),
            "creative" => format!("creative-section-{}", index + 1),
            _ => unreachable!("known catalog scene"),
        };
        let section = screen
            .get_by_test_id(&section_id)
            .rect()
            .expect("chapter geometry");
        // Selecting the sidebar alone is insufficient: the chapter must actually
        // move into the viewport. The final sections clamp at the scroll limit.
        let final_id = match prefix {
            "basic" => format!(
                "basic-section-{}",
                chapters.last().expect("catalogue has chapters")
            ),
            "game" => format!("game-section-{}", chapters.len() - 1),
            "creative" => format!("creative-section-{}", chapters.len()),
            _ => unreachable!("known catalog scene"),
        };
        let end = screen
            .get_by_test_id(&final_id)
            .rect()
            .expect("final chapter geometry");
        let anchor = match prefix {
            "basic" => 76.0,
            "game" => 72.0,
            _ => 100.0,
        };
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
        evidence(screen, &format!("{prefix}-{chapter}"));
    }
}

/// Opt-in leaves ordinary successful tests free of filesystem IO.
fn evidence(screen: &Screen, name: &str) {
    if let Some(directory) = std::env::var_os("KIVACT_CATALOG_EVIDENCE") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("create evidence directory");
        screen
            .screenshot()
            .save_png(directory.join(format!("{name}.png")))
            .expect("save evidence screenshot");
        std::fs::write(
            directory.join(format!("{name}-a11y.txt")),
            screen.accessibility_tree(),
        )
        .expect("save published accessibility tree");
        std::fs::write(
            directory.join(format!("{name}-retained.txt")),
            screen.debug(),
        )
        .expect("save retained semantic tree");
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
    assert_eq!(all.checked(), Some(CheckedState::Indeterminate));
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
