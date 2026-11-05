//! Role: Display and layout component tests: gallery layout and behavior checks.

use kivact::prelude::*;
use kivact::style::Align;
use kivact_ui_basic as basic;

/// A page painted with the `background` token of the theme in effect.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::ColorName::Background)
            .color(basic::ColorName::Foreground)
            .padding(16)
            .gap(16)
            .grow(1)
            .align_items(Align::Start),
    )
}

#[test]
fn pagination_pages_follows_its_binding_and_disables_the_ends() {
    let (screen, page_index) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(basic::theme(kivact::style::ThemeMode::Light))
            .window_size(900.0, 3200.0)
            .reduced_motion(),
        || {
            let index = signal(0usize);
            (
                (index),
                page(
                    kivact::ui::pagination(10usize, index).item(|cell| match cell {
                        PageItem::Page(n) => text((n + 1).to_string()).into_view(),
                        PageItem::Previous => text("Previous").into_view(),
                        PageItem::Next => text("Next").into_view(),
                        _ => text("…").name("More pages").into_view(),
                    }),
                ),
            )
        },
    );
    let previous = || screen.get_by_role(Role::Button, "Previous page");
    let next = || screen.get_by_role(Role::Button, "Next page");
    assert!(previous().is_disabled());
    assert!(!next().is_disabled());
    assert!(screen.get_by_role(Role::Button, "1").is_selected());
    // First, current + 1, last, with one gap between.
    for shown in ["1", "2", "10"] {
        screen.get_by_role(Role::Button, shown);
    }
    assert!(screen.query_by_role(Role::Button, "5").is_none());
    assert_eq!(screen.all_by_text("…").len(), 1);
    next().click();
    assert_eq!(page_index.get_untracked(), 1);
    assert!(!previous().is_disabled());
    assert!(screen.get_by_role(Role::Button, "2").is_selected());
    screen.get_by_role(Role::Button, "10").click();
    assert_eq!(page_index.get_untracked(), 9);
    assert!(next().is_disabled());
    next().force_click();
    assert_eq!(
        page_index.get_untracked(),
        9,
        "a disabled link does not move"
    );
    previous().click();
    assert_eq!(page_index.get_untracked(), 8);
    // Two gaps in the middle; the current page and its neighbours show.
    page_index.set(5);
    screen.run_until_idle();
    for shown in ["1", "5", "6", "7", "10"] {
        screen.get_by_role(Role::Button, shown);
    }
    assert_eq!(screen.all_by_text("…").len(), 2);
    assert!(screen.get_by_role(Role::Button, "6").is_selected());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.assert_layout_clean();
}
