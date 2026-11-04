//! Role: Mount the Soft catalog with searchable navigation and a theme toggle.
use crate::look::canvas;
use crate::model::CatalogState;
use crate::page::page_view;
use crate::pages::pages;
use kivact::control::{Node, NodeBuilder, column, row, scroll, spacer, text};
use kivact::platform::{reduced_motion, set_reduced_motion};
use kivact::style::{Align, ColorToken, Justify, ThemeMode, style};
use kivact::task::spawn;
use kivact::ui::{ButtonVariant, button, text_input};
use kivact::view::{IntoView, dynamic, provide_theme};

/// Mount the catalog with the light appearance.
pub fn catalog() -> impl IntoView {
    catalog_with_mode(ThemeMode::Light)
}

/// Mount the catalog with an explicit initial appearance.
pub fn catalog_with_mode(mode: ThemeMode) -> impl IntoView {
    catalog_at(mode, "button")
}

/// Open the catalog at a named page in the requested appearance.
pub fn catalog_at(mode: ThemeMode, slug: &str) -> impl IntoView {
    let state = CatalogState::new(mode, slug);
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
        text(move || if state.dark.get() { "Dark" } else { "Light" }).test_id("catalog-mode"),
        button("Toggle appearance")
            .test_id("catalog-theme")
            .on_press(move || state.dark.update(|dark| *dark = !*dark)),
        button("Reduce motion")
            .variant(ButtonVariant::Ghost)
            .selected(reduced_motion)
            .test_id("catalog-motion")
            .on_press(|| {
                spawn(set_reduced_motion(Some(!reduced_motion())));
            }),
    ))
    .style(
        style()
            .height(56)
            .shrink(0)
            .gap(8)
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
                            .test_id(page.slug)
                            .variant(ButtonVariant::Ghost)
                            .selected(move || state.selected.get() == index)
                            .style(move || {
                                let base = style().justify_content(Justify::Start);
                                if state.selected.get() == index {
                                    base.background(ColorToken::Hover).weight(600)
                                } else {
                                    base
                                }
                            })
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
