//! Role: Exercise the Soft catalog through its public navigation and appearance controls.
use kivact::test::{RenderOptions, TestFonts, render_with};
use kivact::view::Role;
use kivact_ui_soft_catalog::{catalog, fonts, pages};
use std::sync::Arc;

#[test]
fn search_opens_a_page_and_the_appearance_toggle_keeps_it_open() {
    let screen = render_with(
        RenderOptions {
            fonts: TestFonts::Only(fonts().iter().copied().map(Arc::from).collect()),
            ..RenderOptions::from_env()
        },
        catalog,
    );
    screen.get_by_text("Light");
    let search = screen.get_by_role(Role::TextBox, "Find a component");
    search.type_text("Tooltip");
    screen.get_by_role(Role::Button, "Tooltip").click();
    assert_eq!(screen.get_by_test_id("catalog-title").name(), "Tooltip");
    screen.get_by_test_id("catalog-theme").click();
    screen.get_by_text("Dark");
    assert_eq!(screen.get_by_test_id("catalog-title").name(), "Tooltip");
    assert!(pages().iter().any(|page| page.slug == "rich-text-editor"));
}
