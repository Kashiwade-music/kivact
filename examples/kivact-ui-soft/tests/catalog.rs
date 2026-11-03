//! Role: Exercise the Soft catalog through its public navigation and appearance controls.
use kivact::test::{RenderOptions, TestFonts, render_with};
use kivact::view::{Key, Modifiers, Role};
use kivact_ui_soft_catalog::{catalog, fonts, pages};
use std::sync::Arc;

#[test]
fn every_page_opens_through_search_in_both_appearances() {
    let screen = render_with(
        RenderOptions {
            fonts: TestFonts::Only(fonts().iter().copied().map(Arc::from).collect()),
            ..RenderOptions::from_env()
        },
        catalog,
    );
    let search = screen.get_by_role(Role::TextBox, "Find a component");
    let theme = screen.get_by_test_id("catalog-theme");
    let mode = screen.get_by_test_id("catalog-mode");
    for page in pages() {
        search.press_key(Key::Home);
        search.press_key_with(
            Key::End,
            Modifiers {
                shift: true,
                ..Modifiers::default()
            },
        );
        search.type_text(page.name);
        screen.get_by_test_id(page.slug).click();
        assert_eq!(screen.get_by_test_id("catalog-title").name(), page.name);
        for appearance in ["Light", "Dark"] {
            assert_eq!(mode.text(), appearance);
            let errors = screen.errors();
            assert!(
                errors.is_empty(),
                "{} ({appearance}): {errors:?}",
                page.slug
            );
            theme.click();
        }
    }
}

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
    let motion = screen.get_by_test_id("catalog-motion");
    assert!(!motion.is_selected());
    motion.click();
    assert!(motion.is_selected());
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
