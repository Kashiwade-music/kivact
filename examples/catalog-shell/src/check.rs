//! Role: The page walker of the catalogs' tests: open pages, check layout and glyphs.

use crate::model::Page;
use crate::shell::{CatalogTheme, catalog};
use kivact::prelude::*;
use kivact::test::{LayoutCheck, RenderOptions, Screen, Screenshot, render_with};
use std::time::Duration;

/// Mount `catalog` at 1440×900, scale 1, reduced motion, in `mode`.
pub fn mount(
    title: &'static str,
    pages: fn() -> Vec<Page>,
    theme: CatalogTheme,
    mode: ThemeMode,
) -> Screen {
    render_with(
        RenderOptions::with_theme(theme.get(mode))
            .window_size(1440., 900.)
            .scale(1.)
            .reduced_motion(),
        move || catalog(title, pages(), theme).mode(mode),
    )
}

/// Open every page from the list in both modes: the page is selected, its
/// title is its name, every example has its preview and code, the screen
/// has no errors and no layout violation outside `clipped` (slugs whose
/// previews clip on purpose).
pub fn every_page_opens_cleanly(
    title: &'static str,
    pages: fn() -> Vec<Page>,
    theme: CatalogTheme,
    clipped: &[&str],
) {
    visit_every_page(title, pages, theme, ThemeMode::Light, clipped);
    visit_every_page(title, pages, theme, ThemeMode::Dark, clipped);
}

/// Every visible character above ASCII of the page sources is drawn by one of the theme's fonts.
pub fn every_page_character_is_in_the_fonts(pages: fn() -> Vec<Page>, theme: CatalogTheme) {
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
        RenderOptions::with_theme(theme.get(ThemeMode::Light))
            .window_size(f64::from(COLUMNS * CELL), f64::from(rows * CELL))
            .scale(1.),
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

/// Click `catalog-nav-<slug>` after scrolling it into the list.
pub fn open(screen: &Screen, slug: &str) {
    let id = format!("catalog-nav-{slug}");
    reveal(screen, &id);
    screen.get_by_test_id(&id).click();
    screen.advance(Duration::from_millis(100));
}

fn visit_every_page(
    title: &'static str,
    pages: fn() -> Vec<Page>,
    theme: CatalogTheme,
    mode: ThemeMode,
    clipped: &[&str],
) {
    let all = pages();
    let screen = mount(title, pages, theme, mode);
    let dark = mode == ThemeMode::Dark;
    let label = |slug: &str| {
        if dark {
            format!("dark-{slug}")
        } else {
            slug.to_owned()
        }
    };
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_with(&layout_check(&all, clipped));
    for page in &all {
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
        if !page.usage.is_empty() {
            assert!(screen.query_by_test_id("catalog-usage-code").is_some());
        }
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
        screen.assert_layout_with(&layout_check(&all, clipped));
        screen.evidence(&label(page.slug));
    }
}

/// The layout check that ignores the previews of the `clipped` pages.
fn layout_check(pages: &[Page], clipped: &[&str]) -> LayoutCheck {
    pages
        .iter()
        .filter(|page| clipped.contains(&page.slug))
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

/// The side of a cell of `every_page_character_is_in_the_fonts`.
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
fn cell_pixels(shot: &Screenshot, index: u32) -> Vec<u32> {
    let (x0, y0) = ((index % COLUMNS) * CELL, (index / COLUMNS) * CELL);
    (0..CELL)
        .flat_map(|y| (0..CELL).map(move |x| (x0 + x, y0 + y)))
        .map(|(x, y)| shot.pixel(x, y).expect("the cell is in the window"))
        .collect()
}
