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

#[test]
fn standard_examples_keep_their_controls_when_the_appearance_changes() {
    let screen = render_with(
        RenderOptions {
            fonts: TestFonts::Only(fonts().iter().copied().map(Arc::from).collect()),
            ..RenderOptions::from_env()
        },
        catalog,
    );
    screen.get_by_role(Role::Button, "Primary");
    let search = screen.get_by_role(Role::TextBox, "Find a component");
    search.type_text("Checkbox Input");
    screen.get_by_role(Role::Button, "Checkbox Input").click();
    let checkbox = screen.get_by_role(Role::CheckBox, "Unchecked");
    checkbox.click();
    assert!(
        checkbox
            .checked()
            .is_some_and(|checked| checked.is_checked())
    );
    screen.get_by_test_id("catalog-theme").click();
    assert!(
        checkbox
            .checked()
            .is_some_and(|checked| checked.is_checked())
    );
}
