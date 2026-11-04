//! Role: Render catalog pages and interactions through the standard test backend.
//! The tests open each page from the component list, check the page structure,
//! the example previews and code, the theme toggle, the search and the copy button.
#![warn(redundant_imports)]

use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::test::{LayoutCheck, RenderOptions, Screen, TestFonts, render_with};
use kivact_ui_basic_catalog::{catalog_with_mode, fonts, pages};
use std::time::Duration;

fn mount() -> Screen {
    mount_in(ThemeMode::Light)
}

fn mount_in(mode: ThemeMode) -> Screen {
    render_with(
        RenderOptions::from_env()
            .window_size(1440., 900.)
            .scale(1.)
            .reduced_motion()
            .fonts(TestFonts::Only(
                fonts().iter().map(|&bytes| bytes.into()).collect(),
            )),
        move || catalog_with_mode(mode),
    )
}

/// Pages whose previews show content that a clip hides on purpose: a carousel
/// keeps the slides outside its box, which the check reports as cut text.
const CLIPPED_BY_DESIGN: &[&str] = &["carousel"];

fn layout_check() -> LayoutCheck {
    pages()
        .iter()
        .filter(|page| CLIPPED_BY_DESIGN.contains(&page.slug))
        .flat_map(|page| {
            page.examples
                .iter()
                .map(move |example| format!("example-{}-{}-preview", page.slug, example.id))
        })
        .fold(LayoutCheck::default(), |check, id| check.ignore_test_id(id))
}

fn assert_clean(screen: &Screen, what: &str) {
    assert!(screen.errors().is_empty(), "{what}: {:?}", screen.errors());
    screen.assert_layout_with(&layout_check());
}

/// Scroll the component list until the entry `test_id` is inside it.
fn reveal(screen: &Screen, test_id: &str) {
    let list = screen.get_by_test_id("catalog-nav-scroll");
    let view = list.rect().expect("the component list is laid out");
    for _ in 0..8 {
        let item = screen
            .get_by_test_id(test_id)
            .rect()
            .expect("the entry is laid out");
        if item.y < view.y {
            list.scroll(0., item.y - view.y - 8.);
        } else if item.y + item.height > view.y + view.height {
            list.scroll(0., item.y + item.height - view.y - view.height + 8.);
        } else {
            return;
        }
    }
    panic!("{test_id} did not scroll into the component list");
}

fn open(screen: &Screen, slug: &str) {
    let id = format!("catalog-nav-{slug}");
    reveal(screen, &id);
    screen.get_by_test_id(&id).click();
    screen.advance(Duration::from_millis(100));
}

#[test]
fn every_page_opens_from_the_list_and_renders_cleanly() {
    visit_every_page(ThemeMode::Light);
}

fn visit_every_page(mode: ThemeMode) {
    let screen = mount_in(mode);
    let dark = mode == ThemeMode::Dark;
    let label = |slug: &str| {
        if dark {
            format!("dark-{slug}")
        } else {
            slug.to_owned()
        }
    };
    assert_clean(&screen, "initial page");
    for page in pages() {
        open(&screen, page.slug);
        let nav = screen.get_by_test_id(&format!("catalog-nav-{}", page.slug));
        assert!(nav.is_selected(), "{} is selected in the list", page.slug);
        assert_eq!(
            screen.get_by_test_id("catalog-title").text(),
            page.name,
            "{} title",
            page.slug
        );
        assert!(screen.query_by_test_id("catalog-page").is_some());
        assert!(screen.query_by_test_id("catalog-usage-code").is_some());
        for example in &page.examples {
            for part in ["preview", "code"] {
                let id = format!("example-{}-{}-{part}", page.slug, example.id);
                assert!(
                    screen.query_by_test_id(&id).is_some(),
                    "{id} is on the page"
                );
            }
        }
        assert_clean(&screen, &label(page.slug));
        evidence(&screen, &label(page.slug));
    }
}

/// Opt-in screenshots (`KIVACT_CATALOG_EVIDENCE=<dir>`): ordinary runs do no file IO.
fn evidence(screen: &Screen, name: &str) {
    if let Some(directory) = std::env::var_os("KIVACT_CATALOG_EVIDENCE") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("create evidence directory");
        screen
            .screenshot()
            .save_png(directory.join(format!("{name}.png")))
            .expect("save evidence screenshot");
        // The next screenful of the page, then back to the top.
        let page = screen.get_by_test_id("catalog-page");
        page.scroll(0., 800.);
        screen
            .screenshot()
            .save_png(directory.join(format!("{name}-2.png")))
            .expect("save evidence screenshot");
        page.scroll(0., -800.);
    }
}

#[test]
fn theme_toggle_keeps_the_page_and_the_input_state() {
    let screen = mount();
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
    assert_clean(&screen, "dark input page");
    evidence(&screen, "dark-input");

    screen.get_by_test_id("catalog-theme").click();
    screen.advance(Duration::from_millis(100));
    assert_eq!(screen.screenshot().pixel(300, 800), before);
}

#[test]
fn search_filters_the_list_and_the_keyboard_walks_it() {
    let screen = mount();
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
    assert_clean(&screen, "filtered list");

    search.click();
    for _ in 0..6 {
        search.press_key(Key::Backspace);
    }
    search.type_text("zzzz");
    screen.run_until_idle();
    assert!(screen.query_by_test_id("catalog-nav-dialog").is_none());
    screen.get_by_text("見つかりません");

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
