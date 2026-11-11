//! Role: Render catalog pages and interactions through the standard test backend.
//! The tests open each page from the component list, check the page structure,
//! the example previews and code, the theme toggle_button, the search and the copy button.
#![warn(redundant_imports)]

use kivact::prelude::*;
use kivact::style::Position;
use kivact::test::{LayoutCheck, RenderOptions, Screen, TestFonts, render_with};
use kivact_ui_basic_catalog::{catalog_with_mode, fonts, pages};
use std::time::Duration;

fn mount() -> Screen {
    mount_in(ThemeMode::Light)
}

fn mount_in(mode: ThemeMode) -> Screen {
    render_with(
        RenderOptions::with_theme(kivact_ui_basic::theme(mode))
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
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_with(&layout_check());
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
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
        screen.assert_layout_with(&layout_check());
        screen.evidence(&label(page.slug));
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
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_with(&layout_check());
    screen.evidence("dark-input");

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
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_with(&layout_check());

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

/// The side of a cell of `every_page_character_is_in_the_catalog_fonts`.
const CELL: u32 = 40;
/// Cells per row of that test.
const COLUMNS: u32 = 25;

/// Whether `c` of a page source must be drawn by a catalog font: a visible
/// character above ASCII. Emoji (U+1F000 and above) are left to the
/// system's emoji font, which the catalog does not bundle.
fn bundled(c: char) -> bool {
    !c.is_ascii()
        && !c.is_whitespace()
        && !c.is_control()
        && c < '\u{1F000}'
        && !matches!(c, '\u{200B}'..='\u{200F}' | '\u{2028}'..='\u{202E}' | '\u{2060}'..='\u{206F}' | '\u{FE00}'..='\u{FE0F}' | '\u{FEFF}')
}

/// Cell `index` of the grid: `c` at its top left.
fn glyph_cell(index: u32, c: char) -> View {
    let (column, row) = (index % COLUMNS, index / COLUMNS);
    node()
        .child(text(c.to_string()))
        .style(
            style()
                .position(Position::Absolute)
                .left((column * CELL) as i32)
                .top((row * CELL) as i32)
                .size(CELL as i32, CELL as i32),
        )
        .into_view()
}

/// The pixels of cell `index` of `shot`.
fn cell_pixels(shot: &kivact::test::Screenshot, index: u32) -> Vec<u32> {
    let (x0, y0) = ((index % COLUMNS) * CELL, (index / COLUMNS) * CELL);
    (0..CELL)
        .flat_map(|y| (0..CELL).map(move |x| (x0 + x, y0 + y)))
        .map(|(x, y)| shot.pixel(x, y).expect("the cell is in the window"))
        .collect()
}

/// Every visible character above ASCII of the catalog's pages is drawn by
/// one of the catalog's fonts: none of them draws the missing glyph, which a
/// character no bundled font has gets (the first font's). A character added
/// to a page without running `dev/tools/subset-catalog-font.py` again fails.
#[test]
fn every_page_character_is_in_the_catalog_fonts() {
    let mut chars: Vec<char> = pages()
        .iter()
        .flat_map(|page| page.source.chars())
        .filter(|c| bundled(*c))
        .collect();
    chars.sort_unstable();
    chars.dedup();
    // Cell 0: a character no font has, drawn as the missing glyph.
    let probe = '\u{10FFFD}';
    let cells: Vec<char> = std::iter::once(probe)
        .chain(chars.iter().copied())
        .collect();
    let rows = (cells.len() as u32).div_ceil(COLUMNS);
    let grid = cells.clone();
    let screen = render_with(
        RenderOptions::with_theme(kivact_ui_basic::theme(ThemeMode::Light))
            .window_size(f64::from(COLUMNS * CELL), f64::from(rows * CELL))
            .scale(1.)
            .fonts(TestFonts::Only(
                fonts().iter().map(|&bytes| bytes.into()).collect(),
            )),
        move || {
            let cells: Vec<View> = grid
                .iter()
                .enumerate()
                .map(|(index, c)| glyph_cell(index as u32, *c))
                .collect();
            node().child(cells)
        },
    );
    let shot = screen.screenshot();
    let missing_glyph = cell_pixels(&shot, 0);
    assert!(
        missing_glyph.iter().any(|p| *p != missing_glyph[0]),
        "the missing glyph is drawn"
    );
    let missing: Vec<char> = (1..cells.len() as u32)
        .filter(|index| cell_pixels(&shot, *index) == missing_glyph)
        .map(|index| cells[index as usize])
        .collect();
    assert!(
        missing.is_empty(),
        "no catalog font draws {missing:?}: run dev/tools/subset-catalog-font.py"
    );
}
