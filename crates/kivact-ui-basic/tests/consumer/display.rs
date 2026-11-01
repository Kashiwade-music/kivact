//! Role: Display and layout component tests: gallery layout and behavior checks.

use super::assert_clean;
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::RenderOptions;
use kivact_ui_basic as basic;

/// A page painted with the `background` token of the theme in effect.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::Color::Background)
            .color(basic::Color::Foreground)
            .padding(16)
            .gap(16)
            .grow(1)
            .align_items(Align::Start),
    )
}

fn options(mode: kivact::style::ThemeMode) -> RenderOptions {
    RenderOptions {
        theme: basic::theme(mode),
        width: 900.0,
        height: 3200.0,
        reduced_motion: true,
        ..RenderOptions::from_env()
    }
}

#[test]
fn pagination_pages_follows_its_binding_and_disables_the_ends() {
    let (screen, page_index) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            let index = signal(0usize);
            ((index), page(basic::pagination(index, 10usize)))
        });
    let previous = || screen.get_by_role(Role::Link, "Go to previous page");
    let next = || screen.get_by_role(Role::Link, "Go to next page");
    assert!(previous().is_disabled());
    assert!(!next().is_disabled());
    assert!(screen.get_by_role(Role::Link, "1").is_selected());
    // First, current + 1, last, with one gap between.
    for shown in ["1", "2", "10"] {
        screen.get_by_role(Role::Link, shown);
    }
    assert!(screen.query_by_role(Role::Link, "5").is_none());
    assert_eq!(screen.all_by_role(Role::Text, "More pages").len(), 1);
    next().click();
    assert_eq!(page_index.get_untracked(), 1);
    assert!(!previous().is_disabled());
    assert!(screen.get_by_role(Role::Link, "2").is_selected());
    screen.get_by_role(Role::Link, "10").click();
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
        screen.get_by_role(Role::Link, shown);
    }
    assert_eq!(screen.all_by_role(Role::Text, "More pages").len(), 2);
    assert!(screen.get_by_role(Role::Link, "6").is_selected());
    assert_clean(&screen);
}
