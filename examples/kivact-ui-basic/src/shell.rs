//! Role: The catalog shell: top bar, searchable component list and page area.

use crate::docs::PAGES;
use crate::look::tokens;
use crate::page::page_view;
use kivact::interaction::{ListSelection, use_list};
use kivact::prelude::*;
use kivact_ui_basic as basic;

/// The theme text family: Inter for Latin, Noto Sans JP for Japanese.
const FAMILY: &str = "Inter, 'Noto Sans JP', sans-serif";

const TOP_BAR_HEIGHT: f32 = 56.0;
const LIST_WIDTH: f32 = 256.0;

/// The whole catalog on the kit theme, starting in `mode`.
pub(crate) fn catalog(mode: kivact::style::ThemeMode) -> View {
    let dark = signal(mode == kivact::style::ThemeMode::Dark);
    let theme = move || {
        let theme = basic::theme(if dark.get() {
            kivact::style::ThemeMode::Dark
        } else {
            kivact::style::ThemeMode::Light
        });
        let text = kivact::style::theme::Typography {
            family: Some(FAMILY.into()),
            ..theme.text().clone()
        };
        theme.with_text(text)
    };
    kivact::view::provide_theme(theme, move || {
        let start = PAGES
            .iter()
            .position(|page| page.slug == "button")
            .unwrap_or(0);
        let selected = signal(Some(start));
        column((
            top_bar(dark),
            row((
                component_list(selected),
                column(dynamic(
                    move || selected.get().unwrap_or(start),
                    |index| page_view(&PAGES[index]),
                ))
                .style(style().grow(1).min_width(0).height(pct(100))),
            ))
            .style(style().grow(1).min_height(0)),
            kivact::ui::toast_region(kivact::ui::toaster()).hold_all_on_hover(true),
        ))
        .style(
            style()
                .size(pct(100), pct(100))
                .background(basic::Color::Background)
                .color(basic::Color::Foreground),
        )
    })
}

fn top_bar(dark: Signal<bool>) -> Node {
    let toggle = kivact::ui::button("テーマを切り替え")
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
        text("Kivact UI").style(style().font_size(14).weight(600)),
        text("Basic").style(style().font_size(14).color(basic::Color::MutedForeground)),
        spacer(),
        toggle,
    ))
    .style(
        style()
            .height(TOP_BAR_HEIGHT)
            .shrink(0)
            .padding_x(16)
            .gap(8)
            .align_items(Align::Center)
            .border_bottom(1, basic::Color::Border),
    )
}

/// The search field and the alphabetical, keyboard-navigable list of components.
fn component_list(selected: Signal<Option<usize>>) -> Node {
    let query = signal(String::new());
    let items = (move || {
        let needle = query.get().trim().to_lowercase();
        PAGES
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
    .into_value();
    let nav = use_list(items, ListSelection::Single(selected.into()));
    let rows = for_each(
        items,
        |item| *item.key(),
        move |key, item| {
            let part = nav.item(key);
            let state = part.state();
            row(text(move || item.get().label()).style(style().single_line()))
                .part(part)
                .test_id(format!("catalog-nav-{}", PAGES[key].slug))
                .style(move || {
                    let p = tokens();
                    let state = state.get();
                    let s = style()
                        .height(32)
                        .shrink(0)
                        .padding_x(12)
                        .align_items(Align::Center)
                        .font_size(14)
                        .color(p.foreground);
                    let s = if state.selected {
                        s.background(p.muted).weight(500)
                    } else if state.input.hovered {
                        s.background(p.muted.with_alpha(0.5))
                    } else {
                        s
                    };
                    if state.input.focus_visible {
                        s.ring(1, p.ring.with_alpha(0.5))
                    } else {
                        s
                    }
                })
        },
    );
    let empty = show(
        move || items.with(Vec::is_empty),
        || basic::muted("見つかりません").style(style().padding_x(12).padding_y(8)),
    );
    column((
        column(
            kivact::ui::text_input("コンポーネントを検索", query)
                .placeholder("コンポーネントを検索")
                .test_id("catalog-search"),
        )
        .style(style().padding(12).shrink(0)),
        scroll(
            column((rows, empty))
                .part(nav.list())
                .style(style().padding_x(8).padding_bottom(12)),
        )
        .test_id("catalog-nav-scroll")
        .style(style().grow(1).min_height(0)),
    ))
    .style(
        style()
            .width(LIST_WIDTH)
            .shrink(0)
            .height(pct(100))
            .border_right(1, basic::Color::Border),
    )
}
