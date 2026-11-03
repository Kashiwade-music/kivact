//! Role: Exercise the Soft catalog through its public navigation and appearance controls.
use kivact::style::ThemeMode;
use kivact::test::{RenderOptions, TestFonts, render_with};
use kivact::view::{Key, Modifiers, Role};
use kivact_ui_soft_catalog::{catalog, catalog_at, fonts, pages};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn carousel_example_moves_the_slides_and_reveals_the_previous_control() {
    let screen = render_with(RenderOptions::from_env(), || {
        catalog_at(ThemeMode::Light, "carousel")
    });
    let second = screen.get_by_role(Role::Group, "Slide 2 of 4");
    let before = second.accessibility_bounds().expect("visible slide").x;
    screen.get_by_role(Role::Button, "Scroll right").click();
    screen.advance(Duration::from_millis(800));
    let after = second.accessibility_bounds().expect("scrolled slide").x;
    assert!(after < before, "slide position {before} → {after}");
    assert!(
        !screen
            .get_by_role(Role::Button, "Scroll left")
            .is_disabled()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn carousel_example_snaps_after_horizontal_wheel_input() {
    let screen = render_with(
        RenderOptions {
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || catalog_at(ThemeMode::Light, "carousel"),
    );
    let first = screen.get_by_role(Role::Group, "Slide 1 of 4");
    let start = first.accessibility_bounds().expect("first slide").x;
    let second = screen.get_by_role(Role::Group, "Slide 2 of 4");
    first.scroll(100.0, 0.0);
    let unsettled = second
        .accessibility_bounds()
        .expect("partially scrolled slide")
        .x;
    assert!(unsettled > start + 20.0);
    screen.advance(Duration::from_millis(250));
    let snapped = second.accessibility_bounds().expect("snapped slide").x;
    assert!(
        (snapped - start).abs() < 2.0,
        "slide starts at {snapped}, expected {start}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn table_example_filters_and_expands_projects() {
    let screen = render_with(RenderOptions::from_env(), || {
        catalog_at(ThemeMode::Light, "table")
    });
    screen
        .get_by_role(Role::Button, "Show active projects")
        .click();
    assert!(screen.query_by_role(Role::Row, "Archive").is_none());
    screen
        .get_by_role(Role::Button, "Design details")
        .press_key(Key::Enter);
    screen.get_by_text("Design is Active");
    screen.get_by_role(Role::Button, "Clear filter").click();
    screen.get_by_role(Role::Row, "Archive");
    screen.get_by_text("Design is Active");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

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
