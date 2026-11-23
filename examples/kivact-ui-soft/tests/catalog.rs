//! Exercise the Soft catalog through its page walker and its page controls.
use catalog_shell::check;
use kivact::style::ThemeMode;
use kivact::view::{Key, Role};
use kivact_ui_soft_catalog::{THEME, pages};
use std::time::Duration;

#[test]
fn every_page_opens_from_the_list_and_renders_cleanly() {
    check::every_page_opens_cleanly("Soft", pages, THEME, &["carousel"]);
}

#[test]
fn every_page_character_is_in_the_catalog_fonts() {
    check::every_page_character_is_in_the_fonts(pages, THEME);
}

#[test]
fn carousel_example_moves_the_slides_and_reveals_the_previous_control() {
    let screen = check::mount("Soft", pages, THEME, ThemeMode::Light);
    check::open(&screen, "carousel");
    let second = screen.get_by_role(Role::Group, "Slide 2 / 4");
    let before = second.accessibility_bounds().expect("visible slide").x;
    screen.get_by_role(Role::Button, "Next slide").click();
    screen.advance(Duration::from_millis(800));
    let after = second.accessibility_bounds().expect("scrolled slide").x;
    assert!(after < before, "slide position {before} → {after}");
    assert!(
        !screen
            .get_by_role(Role::Button, "Previous slide")
            .is_disabled()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn carousel_example_snaps_after_horizontal_wheel_input() {
    let screen = check::mount("Soft", pages, THEME, ThemeMode::Light);
    check::open(&screen, "carousel");
    let first = screen.get_by_role(Role::Group, "Slide 1 / 4");
    let start = first.accessibility_bounds().expect("first slide").x;
    let second = screen.get_by_role(Role::Group, "Slide 2 / 4");
    let second_start = second.accessibility_bounds().expect("second slide").x;
    first.scroll(100.0, 0.0);
    let unsettled = second
        .accessibility_bounds()
        .expect("partially scrolled slide")
        .x;
    assert!(unsettled > start + 20.0);
    assert!(unsettled < second_start - 20.0);
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
    let screen = check::mount("Soft", pages, THEME, ThemeMode::Light);
    check::open(&screen, "table");
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
fn standard_examples_keep_their_controls_when_the_appearance_changes() {
    let screen = check::mount("Soft", pages, THEME, ThemeMode::Light);
    check::open(&screen, "checkbox-input");
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

#[test]
fn file_example_uses_picker_storage_and_clear_without_accepting_field_drops() {
    use kivact::geometry::Point;
    use kivact::view::FilePhase;
    let screen = check::mount("Soft", pages, THEME, ThemeMode::Light);
    check::open(&screen, "file-input");
    let field = screen.get_by_role(Role::Button, "Attachment");
    let bounds = field.rect().expect("invariant: file field has layout");
    let point = Point::new(
        bounds.x + bounds.width / 2.0,
        bounds.y + bounds.height / 2.0,
    );
    screen.file_drag(FilePhase::Drop, vec!["ignored.txt".into()], point);
    assert!(screen.query_by_text("ignored.txt").is_none());
    screen.respond_files_dialog(Some(vec!["selected.txt".into()]));
    field.press_key(Key::Enter);
    screen.get_by_text("selected.txt");
    screen.get_by_role(Role::Button, "Clear files").click();
    assert!(screen.query_by_text("selected.txt").is_none());
    assert!(field.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn item_list_examples_keep_native_positions_in_both_appearances() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        for page in ["list", "item"] {
            let screen = check::mount("Soft", pages, THEME, mode);
            check::open(&screen, page);
            let design = screen.get_by_role(Role::ListItem, "Design");
            let archive = screen.get_by_role(Role::ListItem, "Archive");
            assert_eq!(design.pos_in_set(), Some(1));
            assert_eq!(archive.pos_in_set(), Some(3));
            assert_eq!(archive.set_size(), Some(3));
            assert!(screen.errors().is_empty(), "{page}: {:?}", screen.errors());
        }
    }
}
