//! Role: Mount the Soft catalog with searchable navigation and a theme toggle.
use crate::look::canvas;
use crate::model::CatalogState;
use crate::page::page_view;
use crate::pages::pages;
use kivact::control::{Node, NodeBuilder, column, row, scroll, spacer, text};
use kivact::style::{Align, ColorToken, ThemeMode, style};
use kivact::ui::{button, text_input};
use kivact::view::{IntoView, dynamic, provide_theme};

/// Mount the catalog with the light appearance.
pub fn catalog() -> impl IntoView {
    catalog_with_mode(ThemeMode::Light)
}

/// Mount the catalog with an explicit initial appearance.
pub fn catalog_with_mode(mode: ThemeMode) -> impl IntoView {
    let state = CatalogState::new(mode);
    provide_theme(
        move || {
            kivact_ui_soft::theme(if state.dark.get() {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            })
        },
        move || {
            column((
                top_bar(state),
                row((
                    navigation(state),
                    dynamic(
                        move || state.selected.get(),
                        |index| page_view(&pages()[index]),
                    ),
                ))
                .style(style().grow(1).min_height(0)),
            ))
            .style(canvas)
        },
    )
}

fn top_bar(state: CatalogState) -> Node {
    row((
        text("Kivact UI · Soft").style(style().weight(600)),
        spacer(),
        text(move || if state.dark.get() { "Dark" } else { "Light" }),
        button("Toggle appearance")
            .test_id("catalog-theme")
            .on_press(move || state.dark.update(|dark| *dark = !*dark)),
    ))
    .style(
        style()
            .height(56)
            .shrink(0)
            .align_items(Align::Center)
            .padding_x(16)
            .border_bottom(1, ColorToken::Border),
    )
}

fn navigation(state: CatalogState) -> Node {
    column((
        text_input("Find a component", state.query).placeholder("Find a component"),
        scroll(dynamic(
            move || state.query.get(),
            move |query| {
                let query = query.to_lowercase();
                let mut category = "";
                let mut items = Vec::new();
                for (index, page) in pages().iter().enumerate() {
                    if !page.name.to_lowercase().contains(&query) {
                        continue;
                    }
                    if category != page.category {
                        category = page.category;
                        items.push(
                            text(category)
                                .style(style().padding_top(12).weight(600))
                                .into_view(),
                        );
                    }
                    items.push(
                        button(page.name)
                            .on_press(move || state.selected.set(index))
                            .into_view(),
                    );
                }
                column(items).style(style().gap(4))
            },
        ))
        .style(style().grow(1).min_height(0)),
    ))
    .style(
        style()
            .width(224)
            .shrink(0)
            .gap(8)
            .padding(12)
            .border_right(1, ColorToken::Border),
    )
}
