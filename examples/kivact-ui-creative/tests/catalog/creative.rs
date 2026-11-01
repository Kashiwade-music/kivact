//! Role: Public-driver interaction and visual evidence for the complete Creative catalog.
use super::{evidence, mount};
use kivact::draw::Point;
use kivact::test::{Element, Screen};
use kivact::view::{A11yAction, Key, Modifiers, Role};

use std::time::Duration;

fn chapter(screen: &Screen, name: &str) {
    let nav = screen.get_by_test_id(&format!("creative-nav-{name}"));
    let scroll = screen.get_by_test_id("creative-nav-scroll");
    if let (Some(item), Some(view)) = (nav.rect(), scroll.rect()) {
        if item.y + item.height > view.y + view.height {
            scroll.scroll(0.0, item.y + item.height - view.y - view.height + 20.0);
        } else if item.y < view.y {
            scroll.scroll(0.0, item.y - view.y - 20.0);
        }
    }
    nav.click();
    screen.advance(Duration::from_millis(400));
    assert!(nav.is_selected());
    screen.assert_layout_with(&crate::layout_check());
}
fn clean(screen: &Screen) {
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
fn local(screen: &Screen, canvas: &str, region: Element<'_>) -> Point {
    let root = screen.get_by_test_id(canvas).rect().expect("canvas");
    let region = region.rect().expect("region");
    Point::new(region.center().x - root.x, region.center().y - root.y)
}

#[test]
fn creative_picker_hsv_and_alpha_edit_the_same_color() {
    let screen = mount();
    chapter(&screen, "picker");
    let section = screen.get_by_test_id("creative-section-18");
    section.get_by_role(Role::RadioButton, "HSV").click();
    section
        .get_by_role(Role::Slider, "Picker Hue")
        .a11y_with_value(A11yAction::SetValue, "0.3333333333333333");
    section
        .get_by_role(Role::Slider, "Picker Saturation")
        .a11y_with_value(A11yAction::SetValue, "1");
    section
        .get_by_role(Role::Slider, "Picker Value")
        .a11y_with_value(A11yAction::SetValue, "1");
    assert_eq!(
        section
            .get_by_role(Role::TextBox, "HEX Color")
            .value()
            .as_deref(),
        Some("#00FF00")
    );
    let picker = screen
        .get_by_test_id("creative-color-picker")
        .get_by_role(Role::Slider, "Alpha");
    let r = picker.rect().expect("alpha strip");
    let canvas = screen
        .get_by_test_id("creative-color-picker")
        .get_by_role(Role::Canvas, "Color wheel")
        .rect()
        .expect("color canvas");
    picker.drag(
        Point::new(r.center().x - canvas.x, r.center().y - canvas.y),
        Point::new(r.x + r.width * 0.25 - canvas.x, r.center().y - canvas.y),
    );
    assert!(
        (section
            .get_by_role(Role::Slider, "Picker Alpha")
            .value_range()
            .2
            .expect("alpha")
            - 0.25)
            .abs()
            < 0.01
    );
    evidence(&screen, "creative-hsv-alpha");
    chapter(&screen, "properties");
    let location_x = screen
        .get_by_test_id("creative-property-location-x")
        .rect()
        .expect("first property row");
    let location_y = screen
        .get_by_test_id("creative-property-location-y")
        .rect()
        .expect("next property row");
    assert_eq!(location_x.height, 24.0);
    assert_eq!(location_y.y - location_x.y, 27.0);
    let opener = screen
        .get_by_test_id("creative-property-display-color")
        .get_by_role(Role::Button, "Viewport color");
    opener.hover();
    opener.focus();
    let swatch = opener.screenshot();
    assert_eq!(
        swatch.pixel(swatch.width / 2, swatch.height / 2),
        Some(0x00ffffff)
    );
    opener.click();
    let red = || {
        screen
            .get_by_role(Role::Dialog, "Viewport color")
            .get_by_role(Role::SpinButton, "Color R")
    };
    assert_eq!(red().value_range().2, Some(1.0));
    red().a11y_with_value(A11yAction::SetValue, "0.25");
    screen.press_key(Key::Escape);
    opener.click();
    assert_eq!(red().value_range().2, Some(1.0));
    red().a11y_with_value(A11yAction::SetValue, "0.5");
    screen
        .get_by_test_id("creative-section-9")
        .get_by_role(Role::CheckBox, "Name")
        .click();
    assert!(
        screen
            .query_by_role(Role::Dialog, "Viewport color")
            .is_none()
    );
    opener.click();
    assert_eq!(red().value_range().2, Some(0.5));
    evidence(&screen, "creative-properties-color-commit");
    screen.press_key(Key::Escape);
    let swatch = opener.screenshot();
    let pixel = swatch
        .pixel(swatch.width / 2, swatch.height / 2)
        .expect("committed swatch pixel");
    assert!((120..=135).contains(&((pixel >> 16) & 255)));
    assert_eq!(pixel & 0x0000ffff, 0x0000ffff);
    evidence(&screen, "creative-properties-density");
    clean(&screen);
}

#[test]
fn creative_timeline_trim_and_dock_split_are_pointer_operations() {
    let screen = mount();
    chapter(&screen, "timeline");
    let canvas = screen.get_by_test_id("creative-timeline");
    let trim = canvas.get_by_role(Role::Button, "Vox_Take3 end trim");
    let p = local(&screen, "creative-timeline", trim);
    let width = canvas
        .get_by_role(Role::Button, "Vox_Take3")
        .rect()
        .expect("clip")
        .width;
    trim.drag(p, Point::new(p.x + 30.0, p.y));
    assert!(
        (canvas
            .get_by_role(Role::Button, "Vox_Take3")
            .rect()
            .expect("clip")
            .width
            - width
            - 30.0)
            .abs()
            < 0.1
    );
    evidence(&screen, "creative-timeline-trim");
    chapter(&screen, "docking");
    let workspace = screen.get_by_test_id("creative-dock-workspace");
    let splitter = workspace.get_by_role(Role::Splitter, "Info and Text Editor");
    let rect = splitter.rect().expect("splitter");
    splitter.drag(Point::new(3.0, 100.0), Point::new(43.0, 100.0));
    assert!(splitter.rect().expect("moved splitter").x > rect.x + 30.0);
    let editor = workspace.get_by_role(Role::TextBox, "Document source");
    let console = workspace.get_by_role(Role::Group, "Console");
    let editor_height = editor.rect().expect("editor").height;
    let console_height = console.rect().expect("console").height;
    let split = workspace.get_by_role(Role::Splitter, "Resize");
    split.drag(Point::new(50.0, 3.0), Point::new(50.0, -17.0));
    assert!((editor.rect().expect("editor").height - editor_height + 20.0).abs() < 0.5);
    assert!((console.rect().expect("console").height - console_height - 20.0).abs() < 0.5);
    workspace
        .get_by_role(Role::Tab, "Text Editor")
        .press_key_with(
            Key::Space,
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
        );
    assert!(!splitter.is_visible());
    workspace
        .get_by_role(Role::Tab, "Text Editor")
        .press_key(Key::Escape);
    assert!(splitter.is_visible());
    evidence(&screen, "creative-dock-restored");
    clean(&screen);
}
