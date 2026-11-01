//! Role: Every page of the document-style catalog renders cleanly on a real window.
//! The tests open each page from the component list, check the page structure,
//! the example previews and code, the theme toggle, the search and the copy button.
#![warn(redundant_imports)]

use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::test::{
    Element, LayoutCheck, RenderOptions, Screen, TestFonts, WindowMode, render_with,
};
use kivact_ui_basic_catalog::{catalog_with_mode, fonts, pages};
use std::sync::Arc;
use std::time::Duration;

fn mount() -> Screen {
    mount_in(ThemeMode::Light)
}

fn mount_in(mode: ThemeMode) -> Screen {
    let gpu = std::env::var("KIVACT_CATALOG_GPU").is_ok_and(|value| value == "1");
    render_with(
        RenderOptions {
            width: 1440.,
            height: 900.,
            scale: 1.,
            reduced_motion: true,
            window: WindowMode::Real,
            gpu,
            fonts: TestFonts::Only(fonts().iter().map(|&bytes| Arc::from(bytes)).collect()),
            ..RenderOptions::from_env()
        },
        move || catalog_with_mode(mode),
    )
}

/// The names of the functions `source` defines (`fn name`).
fn defined_fns(source: &str) -> Vec<&str> {
    source
        .split("fn ")
        .skip(1)
        .filter_map(|rest| {
            let end = rest.find(|c: char| !(c.is_alphanumeric() || c == '_'))?;
            (end > 0).then(|| &rest[..end])
        })
        .collect()
}

/// Whether `snippet` calls the free function `name` (not a method or a
/// path such as `ui::name`).
fn calls(snippet: &str, name: &str) -> bool {
    let call = format!("{name}(");
    snippet.match_indices(&call).any(|(at, _)| {
        let before = snippet[..at].chars().next_back();
        !before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == ':')
    })
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
fn every_example_has_a_snippet_of_real_code() {
    let pages = pages();
    assert_eq!(pages.len(), 66, "all component slugs are listed");
    let slugs = pages.iter().map(|page| page.slug).collect::<Vec<_>>();
    let mut sorted = slugs.clone();
    sorted.sort_unstable();
    assert_eq!(slugs, sorted, "pages are alphabetical");
    for page in &pages {
        assert!(!page.examples.is_empty(), "{} has examples", page.slug);
        for example in &page.examples {
            assert!(
                !example.snippet.trim().is_empty(),
                "{}/{} has a snippet",
                page.slug,
                example.id
            );
            assert!(
                example.snippet.contains("fn"),
                "{}/{} shows a function",
                page.slug,
                example.id
            );
            assert!(
                !example.snippet.contains("// end example")
                    && !example.snippet.contains("// example:"),
                "{}/{} keeps the markers out of the snippet",
                page.slug,
                example.id
            );
            let own = defined_fns(&example.snippet);
            for name in defined_fns(page.source) {
                assert!(
                    own.contains(&name) || !calls(&example.snippet, name),
                    "{}/{} calls `{name}` without defining it in the snippet",
                    page.slug,
                    example.id
                );
            }
        }
    }
}

#[test]
fn every_page_opens_from_the_list_and_renders_cleanly() {
    visit_every_page(ThemeMode::Light);
}

#[test]
fn every_page_renders_cleanly_in_the_dark_theme() {
    visit_every_page(ThemeMode::Dark);
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

#[test]
fn copy_button_writes_the_example_source_to_the_clipboard() {
    let screen = mount();
    open(&screen, "button");
    let page = pages()
        .into_iter()
        .find(|page| page.slug == "button")
        .expect("the button page exists");
    let example = &page.examples[0];
    let code = screen.get_by_test_id(&format!("example-button-{}-code", example.id));
    screen
        .within(code)
        .get_by_role(Role::Button, "Copy")
        .click();
    screen.advance(Duration::from_millis(100));
    assert_eq!(
        screen.clipboard().as_deref(),
        Some(example.snippet.as_str())
    );
    assert_clean(&screen, "after copy");
}

/// Open the overlay demo `slug`/`id` through its trigger and check that the
/// overlay it shows is named `name` and renders cleanly.
fn open_demo(screen: &Screen, slug: &str, id: &str, trigger: &str, role: Role, name: &str) {
    open(screen, slug);
    let button = demo_button(screen, slug, id, trigger);
    button.click();
    screen.advance(Duration::from_millis(100));
    screen.get_by_role(role, name);
    assert_clean(screen, &format!("{slug}/{id} opened"));
}

#[test]
fn overlay_demos_open_from_their_triggers_and_render_cleanly() {
    let screen = mount();
    for (slug, id, trigger, role, name) in [
        (
            "alert-dialog",
            "default",
            "Show Dialog",
            Role::AlertDialog,
            "Are you absolutely sure?",
        ),
        (
            "alert-dialog",
            "destructive",
            "Delete Chat",
            Role::AlertDialog,
            "Delete chat?",
        ),
        (
            "alert-dialog",
            "with-media",
            "Share Chat",
            Role::AlertDialog,
            "Share this chat?",
        ),
        (
            "alert-dialog",
            "small",
            "Show Dialog",
            Role::AlertDialog,
            "Allow accessory to connect?",
        ),
        ("sheet", "default", "Open", Role::Dialog, "Edit profile"),
        (
            "sheet",
            "no-close-button",
            "Open",
            Role::Dialog,
            "Terms of service",
        ),
        (
            "drawer",
            "default",
            "Open Drawer",
            Role::Dialog,
            "Move Goal",
        ),
        (
            "popover",
            "default",
            "Open popover",
            Role::Dialog,
            "Dimensions",
        ),
        (
            "command",
            "default",
            "Open command menu",
            Role::Dialog,
            "Command palette",
        ),
    ] {
        open_demo(&screen, slug, id, trigger, role, name);
        screen.press_key(Key::Escape);
        screen.advance(Duration::from_millis(100));
        assert!(
            screen.query_by_role(role, name).is_none(),
            "{slug}/{id}: Escape closes it"
        );
    }
}

/// The button `name` in the preview of the demo `slug`/`id`, scrolled into view.
fn demo_button<'a>(screen: &'a Screen, slug: &str, id: &str, name: &str) -> Element<'a> {
    let preview = screen.get_by_test_id(&format!("example-{slug}-{id}-preview"));
    let button = screen.within(preview).get_by_role(Role::Button, name);
    let page = screen.get_by_test_id("catalog-page");
    let rect = button.rect().expect("the demo button is laid out");
    if rect.y < 70.0 || rect.y + rect.height > 880.0 {
        page.scroll(0., rect.y - 450.0);
    }
    button
}

#[test]
fn accordion_toast_and_hover_card_demos_respond_to_the_pointer() {
    let screen = mount();
    open(&screen, "accordion");
    demo_button(&screen, "accordion", "default", "Shipping Details").click();
    screen.advance(Duration::from_millis(100));
    screen.get_by_text("We offer worldwide shipping through trusted courier partners.");
    assert!(
        screen
            .query_by_text(
                "Our flagship product combines cutting-edge technology with sleek design."
            )
            .is_none(),
        "opening one section closes the other"
    );
    assert_clean(&screen, "accordion opened");

    open(&screen, "toast");
    for (label, role) in [("Success", Role::Status), ("Error", Role::Alert)] {
        demo_button(&screen, "toast", "variants", label).click();
        screen.advance(Duration::from_millis(100));
        screen.get_by_role(role, &format!("{label} toast"));
    }
    assert_clean(&screen, "toasts shown");
    screen.advance(Duration::from_secs(6));
    assert!(
        screen
            .query_by_role(Role::Status, "Success toast")
            .is_none()
    );

    open(&screen, "hover-card");
    demo_button(&screen, "hover-card", "default", "@nextjs").hover();
    screen.advance(Duration::from_millis(700));
    screen.get_by_role(Role::Group, "Preview");
    assert_clean(&screen, "hover card opened");
}

#[test]
fn example_code_is_highlighted_and_a_selection_is_copied_with_ctrl_c() {
    let screen = mount();
    open(&screen, "button");
    let page = pages()
        .into_iter()
        .find(|page| page.slug == "button")
        .expect("the button page exists");
    let example = &page.examples[0];
    let block = screen.get_by_test_id(&format!("example-button-{}-code", example.id));
    let code = screen.within(block).get_by_role(Role::TextBox, "Rust");
    assert!(code.read_only());
    assert_eq!(code.value().as_deref(), Some(example.snippet.as_str()));
    // The keywords are drawn in the theme's keyword color.
    let keyword = kivact_ui_basic::CodeColors::new(ThemeMode::Light).keyword;
    let [r, g, b, _] = keyword.components();
    let channel = |v: f32| (v * 255.0).round() as i32;
    let keyword = [channel(r), channel(g), channel(b)];
    let rect = code.rect().expect("laid out");
    let shot = screen.screenshot();
    let near = (rect.y as u32..(rect.y + 23.0) as u32).any(|y| {
        (rect.x as u32..(rect.x + rect.width) as u32).any(|x| {
            shot.pixel(x, y).is_some_and(|p| {
                let p = [
                    (p >> 16) as i32 & 0xff,
                    (p >> 8) as i32 & 0xff,
                    p as i32 & 0xff,
                ];
                (0..3).all(|i| (p[i] - keyword[i]).abs() <= 24)
            })
        })
    });
    assert!(near, "the first line's `fn` is in the keyword color");
    // A part of the code: the first line, selected by keyboard.
    code.focus();
    code.press_key_with(
        Key::Home,
        kivact::view::Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    code.press_key_with(
        Key::End,
        kivact::view::Modifiers {
            shift: true,
            ..Default::default()
        },
    );
    code.press_key_with(
        Key::Char('c'),
        kivact::view::Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    let first = example.snippet.lines().next().expect("a line");
    assert_eq!(screen.clipboard().as_deref(), Some(first));
    assert_clean(&screen, "after selecting the code");
}

#[test]
fn a_growing_demo_keeps_its_first_row_in_place() {
    let screen = mount();
    open(&screen, "accordion");
    let first = |screen: &Screen| {
        demo_button(screen, "accordion", "default", "Product Information")
            .rect()
            .expect("laid out")
            .y
    };
    let preview = |screen: &Screen| {
        screen
            .get_by_test_id("example-accordion-default-preview")
            .rect()
            .expect("laid out")
            .y
    };
    let before = first(&screen) - preview(&screen);
    assert_eq!(
        before, 41.0,
        "at the top of the box, inside its border and padding"
    );

    for name in ["Product Information", "Shipping Details", "Return Policy"] {
        demo_button(&screen, "accordion", "default", name).click();
        screen.advance(Duration::from_millis(100));
        assert_eq!(
            first(&screen) - preview(&screen),
            before,
            "the first row stays put after {name} toggles"
        );
    }
    assert_clean(&screen, "accordion toggled");
}
