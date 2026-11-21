//! Role: The catalog shell: top bar, searchable component list and page area.

use crate::fonts::FONTS;
use crate::model::Page;
use crate::page::page_view;
use kivact::interaction::{ListOptions, Selection, use_list};
use kivact::prelude::*;
use kivact::state::{IntoValue, try_use_context};
use kivact::view::Node;
use std::rc::Rc;

const TOP_BAR_HEIGHT: f64 = 56.0;
const LIST_WIDTH: f64 = 256.0;

/// The icons of the shell's theme toggle, which every kit's catalog draws
/// (Lucide, ISC; the license is in each file).
const MOON: &[u8] = include_bytes!("../assets/icons/moon.svg");
const SUN: &[u8] = include_bytes!("../assets/icons/sun.svg");

/// A catalog's theme: the theme function of its kit (the colors and the font
/// family of the catalog), with the catalog's fonts and the shell's icons.
#[derive(Clone, Copy)]
pub struct CatalogTheme(fn(ThemeMode) -> Theme);

impl CatalogTheme {
    /// The catalog theme of `theme`, which returns the kit's theme for a mode.
    pub const fn new(theme: fn(ThemeMode) -> Theme) -> CatalogTheme {
        CatalogTheme(theme)
    }

    /// The catalog's theme in `mode`, with the catalog fonts and the shell's icons.
    pub fn get(&self, mode: ThemeMode) -> Theme {
        (self.0)(mode)
            .with_fonts(FONTS.iter().copied())
            .with_icon("moon", MOON)
            .with_icon("sun", SUN)
    }
}

/// The catalog titled `title` with `pages` in order, shown in `theme`.
/// `start` is the slug of the page shown first (the first page if none).
#[component]
pub fn catalog(
    title: &'static str,
    pages: Vec<Page>,
    theme: CatalogTheme,
    #[prop(default = "")] start: &'static str,
    #[prop(default = ThemeMode::Light)] mode: ThemeMode,
) -> impl IntoView {
    let pages = Rc::new(pages);
    let start = pages
        .iter()
        .position(|page| page.slug == start)
        .unwrap_or(0);
    let dark = signal(mode == ThemeMode::Dark);
    let themed = move || {
        theme.get(if dark.get() {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        })
    };
    provide_theme(themed, move || {
        let selected = signal(Some(start));
        let nav = PageNav {
            selected,
            pages: pages.clone(),
        };
        provide(nav, move || {
            column((
                top_bar(title, dark),
                row((
                    component_list(pages.clone(), selected),
                    page_area(pages, selected, start),
                ))
                .style(style().flex_grow(1).min_height(0)),
                kivact::ui::toast_region(kivact::interaction::use_toaster())
                    .hold_all_on_hover(true),
            ))
            .style(
                style()
                    .size(pct(100), pct(100))
                    .background(color::SURFACE)
                    .color(color::TEXT),
            )
        })
    })
}

/// The page selection of the catalog on screen, which [`open_page`] reads.
#[derive(Clone)]
struct PageNav {
    selected: Signal<Option<usize>>,
    pages: Rc<Vec<Page>>,
}

/// Open the page `slug` of the catalog on screen, as its list entry does, from
/// a control inside a page. It does nothing when no catalog is shown or when no
/// page has `slug`.
pub fn open_page(slug: &str) {
    let Some(nav) = try_use_context::<PageNav>() else {
        return;
    };
    if let Some(index) = nav.pages.iter().position(|page| page.slug == slug) {
        nav.selected.set(Some(index));
    }
}

/// The scrolling page of the selected entry.
fn page_area(pages: Rc<Vec<Page>>, selected: Signal<Option<usize>>, start: usize) -> Node {
    column(dynamic(
        move || selected.get().unwrap_or(start),
        move |index| page_view(&pages[index]),
    ))
    .style(style().flex_grow(1).min_width(0).height(pct(100)))
}

fn top_bar(title: &'static str, dark: Signal<bool>) -> Node {
    let toggle = kivact::ui::button("Toggle theme")
        .variant(kivact::ui::ButtonVariant::Outline)
        .hide_label(true)
        .icon(dynamic(
            move || dark.get(),
            |dark| icon(if dark { "sun" } else { "moon" }),
        ))
        .style(style().aspect_ratio(1.0).padding_x(0))
        .test_id("catalog-theme")
        .on_press(move || dark.update(|dark| *dark = !*dark));
    row((
        text("Kivact UI").style(style().font_size(14).font_weight(600)),
        text(title).style(style().font_size(14).color(color::TEXT_MUTED)),
        spacer(),
        toggle,
    ))
    .style(
        style()
            .height(TOP_BAR_HEIGHT)
            .flex_shrink(0)
            .padding_x(16)
            .gap(8)
            .align_items(Align::Center)
            .border_bottom(1, color::BORDER),
    )
}

/// The entries that match `query` by name or slug, in page order.
fn component_items(pages: Rc<Vec<Page>>, query: Signal<String>) -> Value<Vec<ListItem<usize>>> {
    (move || {
        let needle = query.get().trim().to_lowercase();
        pages
            .iter()
            .enumerate()
            .filter(|(_, page)| {
                needle.is_empty()
                    || page.name.to_lowercase().contains(&needle)
                    || page.slug.contains(&needle)
            })
            .map(|(index, page)| ListItem::new(index, page.name))
            .collect::<Vec<_>>()
    })
    .into_value()
}

/// Whether the entry `key` starts a category among the entries shown.
fn starts_group(pages: &[Page], shown: &[ListItem<usize>], key: usize) -> bool {
    let Some(at) = shown.iter().position(|item| *item.key() == key) else {
        return false;
    };
    at == 0 || pages[*shown[at - 1].key()].category != pages[key].category
}

/// The category heading above the entry `key`, where a category starts.
fn group_heading(
    pages: Rc<Vec<Page>>,
    items: Value<Vec<ListItem<usize>>>,
    key: usize,
) -> impl IntoView {
    let category = pages[key].category;
    show(
        move || starts_group(&pages, &items.get(), key),
        move || {
            text(category).style(
                style()
                    .font_size(12)
                    .color(color::TEXT_MUTED)
                    .padding_x(12)
                    .padding_top(8),
            )
        },
    )
}

/// The search field and the category-grouped, keyboard-navigable list of pages.
fn component_list(pages: Rc<Vec<Page>>, selected: Signal<Option<usize>>) -> Node {
    let query = signal(String::new());
    let items = component_items(pages.clone(), query);
    let nav = use_list(items, Selection::Single(selected), ListOptions::default());
    let rows = for_each(
        items,
        |item| *item.key(),
        move |item| {
            let key = *item.get().key();
            let slug = pages[key].slug;
            let part = nav.item(key);
            let state = part.state();
            column((
                group_heading(pages.clone(), items, key),
                row(text(move || item.get().label()).style(style().single_line()))
                    .part(part)
                    .test_id(format!("catalog-nav-{slug}"))
                    .style(move || {
                        let state = state.get();
                        let s = style()
                            .height(32)
                            .flex_shrink(0)
                            .padding_x(12)
                            .align_items(Align::Center)
                            .font_size(14)
                            .color(color::TEXT);
                        let s = if state.selected {
                            s.background(color::SURFACE_ALT).font_weight(500)
                        } else if state.input.is_hovered() {
                            s.background(color::SURFACE_ALT.with_alpha(0.5))
                        } else {
                            s
                        };
                        if state.input.is_focus_visible() {
                            s.outline(1, color::FOCUS.with_alpha(0.5))
                        } else {
                            s
                        }
                    }),
            ))
        },
    );
    let empty = show(
        move || items.with(Vec::is_empty),
        || {
            text("No results").style(
                style()
                    .padding_x(12)
                    .padding_y(8)
                    .font_size(14)
                    .color(color::TEXT_MUTED),
            )
        },
    );
    column((
        column(
            kivact::ui::text_input(query)
                .label("Search components")
                .placeholder("Search components")
                .test_id("catalog-search"),
        )
        .style(style().padding(12).flex_shrink(0)),
        scroll(
            column((rows, empty))
                .part(nav.list())
                .style(style().padding_x(8).padding_bottom(12)),
        )
        .test_id("catalog-nav-scroll")
        .style(style().flex_grow(1).min_height(0)),
    ))
    .style(
        style()
            .width(LIST_WIDTH)
            .flex_shrink(0)
            .height(pct(100))
            .border_right(1, color::BORDER),
    )
}
