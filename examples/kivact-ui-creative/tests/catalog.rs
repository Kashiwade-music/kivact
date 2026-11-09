//! Role: All source chapters remain reachable in the complete, font-matched shell.
//! Optional artifacts are produced by the same public test driver with optional window mirroring
//! that consumers use; no library reaches into an implementation crate.
//! Public-driver interaction and visual evidence for the complete Creative catalog.
#![warn(redundant_imports)]

use kivact::geometry::Point;
use kivact::test::{LayoutCheck, Locator, RenderOptions, Screen, TestFonts, render_with};
use kivact::view::{Key, Modifiers, Role};
use kivact_ui_creative_catalog::{catalog, fonts};
use std::time::Duration;

#[test]
fn creative_source_editors_scroll_in_the_original_shell() {
    let screen = mount();
    screen.evidence("creative-initial");
    chapters(
        &screen,
        "creative",
        &[
            "buttons",
            "numeric",
            "selection",
            "indicators",
            "transport",
            "knobs",
            "meters",
            "mixer",
            "properties",
            "docking",
            "outliner",
            "media",
            "layers",
            "timeline",
            "animation",
            "nodes",
            "grading",
            "picker",
            "viewport",
            "menus",
            "dialogs",
            "jobs",
        ],
    );
}

fn mount() -> Screen {
    render_with(
        RenderOptions::from_env()
            .window_size(1861., 969.)
            .scale(1.)
            .reduced_motion()
            .fonts(TestFonts::Only(
                fonts().iter().map(|&bytes| bytes.into()).collect(),
            )),
        catalog,
    )
}

/// Preserve the catalog layout exceptions. The "Z" axis labels of the inspector stick out of
/// their box by 1px on purpose and are ignored by test id.
fn layout_check() -> LayoutCheck {
    LayoutCheck::default().ignore_test_id("axis-label-z")
}

fn chapters(screen: &Screen, prefix: &str, chapters: &[&str]) {
    for (index, chapter) in chapters.iter().enumerate() {
        let nav = screen.get_by_test_id(&format!("{prefix}-nav-{chapter}"));
        if prefix == "creative" {
            let scroll = screen.get_by_test_id("creative-nav-scroll");
            let viewport = scroll.rect().expect("sidebar viewport");
            let item = nav.rect().expect("sidebar item");
            if item.y + item.height > viewport.y + viewport.height {
                scroll.scroll(
                    0.,
                    item.y + item.height - viewport.y - viewport.height + 12.,
                );
            }
        }
        nav.click();
        screen.advance(Duration::from_millis(400));
        assert!(nav.is_selected(), "{prefix}/{chapter} remains selected");
        let section_id = match prefix {
            "basic" => format!("basic-section-{chapter}"),
            "game" => format!("game-section-{index}"),
            "creative" => format!("creative-section-{}", index + 1),
            _ => unreachable!("known catalog scene"),
        };
        let section = screen
            .get_by_test_id(&section_id)
            .rect()
            .expect("chapter geometry");
        // Selecting the sidebar alone is insufficient: the chapter must actually
        // move into the viewport. The final sections clamp at the scroll limit.
        let final_id = match prefix {
            "basic" => format!(
                "basic-section-{}",
                chapters.last().expect("catalogue has chapters")
            ),
            "game" => format!("game-section-{}", chapters.len() - 1),
            "creative" => format!("creative-section-{}", chapters.len()),
            _ => unreachable!("known catalog scene"),
        };
        let end = screen
            .get_by_test_id(&final_id)
            .rect()
            .expect("final chapter geometry");
        let anchor = match prefix {
            "basic" => 76.0,
            "game" => 72.0,
            _ => 100.0,
        };
        let at_scroll_end = section.y > anchor + 8.0 && end.y + end.height <= 971.0;
        if at_scroll_end {
            assert!(
                (0.0..=600.0).contains(&section.y),
                "{prefix}/{chapter} visible at scroll limit: {section:?}"
            );
        } else {
            assert!(
                (section.y - anchor).abs() <= 8.0,
                "{prefix}/{chapter} scrolls to {anchor}: {section:?}"
            );
        }
        assert!(
            screen.errors().is_empty(),
            "{prefix}/{chapter}: {:?}",
            screen.errors()
        );
        screen.assert_layout_with(&layout_check());
        screen.evidence(&format!("{prefix}-{chapter}"));
    }
}

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
    screen.assert_layout_with(&layout_check());
}

fn local(screen: &Screen, canvas: &str, region: Locator<'_>) -> Point {
    let root = screen.get_by_test_id(canvas).rect().expect("canvas");
    let region = region.rect().expect("region");
    Point::new(region.center().x - root.x, region.center().y - root.y)
}

#[test]
fn creative_picker_hsv_and_alpha_edit_the_same_color() {
    let screen = mount();
    assert_picker_hsv_and_alpha(&screen);
    assert_property_color_editing(&screen);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn assert_picker_hsv_and_alpha(screen: &Screen) {
    chapter(screen, "picker");
    let section = screen.get_by_test_id("creative-section-18");
    section.get_by_role(Role::RadioButton, "HSV").click();
    section
        .get_by_role(Role::Slider, "Picker Hue")
        .a11y_set_value("0.3333333333333333");
    section
        .get_by_role(Role::Slider, "Picker Saturation")
        .a11y_set_value("1");
    section
        .get_by_role(Role::Slider, "Picker Value")
        .a11y_set_value("1");
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
    screen.evidence("creative-hsv-alpha");
}

fn assert_property_color_editing(screen: &Screen) {
    chapter(screen, "properties");
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
    red().a11y_set_value("0.25");
    screen.press_key(Key::Escape);
    opener.click();
    assert_eq!(red().value_range().2, Some(1.0));
    red().a11y_set_value("0.5");
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
    screen.evidence("creative-properties-color-commit");
    screen.press_key(Key::Escape);
    let swatch = opener.screenshot();
    let pixel = swatch
        .pixel(swatch.width / 2, swatch.height / 2)
        .expect("committed swatch pixel");
    assert!((120..=135).contains(&((pixel >> 16) & 255)));
    assert_eq!(pixel & 0x0000ffff, 0x0000ffff);
    screen.evidence("creative-properties-density");
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
    screen.evidence("creative-timeline-trim");
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
    screen.evidence("creative-dock-restored");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
