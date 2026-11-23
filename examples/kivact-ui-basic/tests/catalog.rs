//! Role: The Basic catalog's tests: the page walker, the theme toggle, the search
//! and the keyboard walk of the list.

use catalog_shell::check::{
    every_page_character_is_in_the_fonts, every_page_opens_cleanly, mount, open,
};
use kivact::prelude::*;
use kivact::test::LayoutCheck;
use kivact_ui_basic_catalog::{THEME, pages};
use std::time::Duration;

#[test]
fn every_page_opens_from_the_list_and_renders_cleanly() {
    every_page_opens_cleanly("Basic", pages, THEME, &["carousel"]);
}

#[test]
fn every_page_character_is_in_the_catalog_fonts() {
    every_page_character_is_in_the_fonts(pages, THEME);
}

#[test]
fn theme_toggle_keeps_the_page_and_the_input_state() {
    let screen = mount("Basic", pages, THEME, ThemeMode::Light);
    let search = screen.get_by_test_id("catalog-search");
    search.click();
    search.type_text("in");
    open(&screen, "input");
    let preview = screen.get_by_test_id("example-input-basic-preview");
    let demo = screen.within(preview).get_by_role(Role::TextBox, "Email");
    demo.click();
    demo.type_text("ada@example.com");
    assert_eq!(demo.value().as_deref(), Some("ada@example.com"));
    let before = screen.screenshot().pixel(300, 800);

    screen.get_by_test_id("catalog-theme").click();
    screen.advance(Duration::from_millis(100));
    assert_eq!(screen.get_by_test_id("catalog-title").text(), "Input");
    assert!(screen.get_by_test_id("catalog-nav-input").is_selected());
    assert_eq!(
        demo.value().as_deref(),
        Some("ada@example.com"),
        "the demo keeps its text across the theme change"
    );
    assert_eq!(
        screen.get_by_test_id("catalog-search").value().as_deref(),
        Some("in")
    );
    assert_ne!(
        screen.screenshot().pixel(300, 800),
        before,
        "the page is restyled in place"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_with(&LayoutCheck::default());
    screen.evidence("dark-input");

    screen.get_by_test_id("catalog-theme").click();
    screen.advance(Duration::from_millis(100));
    assert_eq!(screen.screenshot().pixel(300, 800), before);
}

#[test]
fn search_filters_the_list_and_the_keyboard_walks_it() {
    let screen = mount("Basic", pages, THEME, ThemeMode::Light);
    let search = screen.get_by_test_id("catalog-search");
    search.click();
    search.type_text("dialog");
    screen.run_until_idle();
    assert!(screen.query_by_test_id("catalog-nav-dialog").is_some());
    assert!(
        screen
            .query_by_test_id("catalog-nav-alert-dialog")
            .is_some()
    );
    assert!(screen.query_by_test_id("catalog-nav-button").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_with(&LayoutCheck::default());

    search.click();
    for _ in 0..6 {
        search.press_key(Key::Backspace);
    }
    search.type_text("zzzz");
    screen.run_until_idle();
    assert!(screen.query_by_test_id("catalog-nav-dialog").is_none());
    screen.get_by_text("No results");

    search.click();
    for _ in 0..4 {
        search.press_key(Key::Backspace);
    }
    screen.run_until_idle();
    assert!(screen.query_by_test_id("catalog-nav-button").is_some());

    open(&screen, "button");
    let button = screen.get_by_test_id("catalog-nav-button");
    button.focus();
    button.press_key(Key::ArrowDown);
    screen.advance(Duration::from_millis(100));
    assert_eq!(
        screen.get_by_test_id("catalog-title").text(),
        "Button Group",
        "ArrowDown selects the next component"
    );
}
