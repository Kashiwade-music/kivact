//! Role: Public-driver interaction and visual evidence for the complete Creative catalog.
use super::{evidence, mount};
use kivact::draw::Point;
use kivact::test::{Element, Screen};
use kivact::view::{A11yAction, Key, Modifiers, MouseButton, Role};

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
fn fill(input: Element<'_>, value: &str) {
    input.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    input.type_text(value);
}
fn local(screen: &Screen, canvas: &str, region: Element<'_>) -> Point {
    let root = screen.get_by_test_id(canvas).rect().expect("canvas");
    let region = region.rect().expect("region");
    Point::new(region.center().x - root.x, region.center().y - root.y)
}

#[test]
fn creative_last_nav_item_scrolls_into_the_sidebar_and_is_fully_visible() {
    let screen = mount();
    let scroll = screen.get_by_test_id("creative-nav-scroll");
    let last = screen.get_by_test_id("creative-nav-jobs");
    let view = scroll.rect().expect("sidebar scroll");
    let before = last.rect().expect("last nav item");
    assert!(
        before.y + before.height > view.y + view.height,
        "the sidebar has more rows than it shows: {before:?} in {view:?}"
    );
    scroll.scroll(0.0, before.y + before.height - view.y - view.height + 20.0);
    screen.advance(Duration::from_millis(400));
    let after = last.rect().expect("last nav item");
    assert!(
        after.y >= view.y && after.y + after.height <= view.y + view.height,
        "the last nav item is fully inside the sidebar: {after:?} in {view:?}"
    );
}

#[test]
fn creative_lime_compact_numeric_and_button_states() {
    let screen = mount();
    screen.get_by_role(Role::Button, "Accent Lime").click();
    screen.get_by_role(Role::Button, "Compact density").click();
    assert!(
        screen
            .get_by_role(Role::Button, "Compact density")
            .is_selected()
    );
    chapter(&screen, "numeric");
    let section = screen.get_by_test_id("creative-section-2");
    let exposure = section.get_by_role(Role::SpinButton, "Exposure");
    assert_eq!(exposure.rect().map(|r| r.height), Some(20.0));
    exposure.click();
    fill(section.get_by_role(Role::TextBox, "Exposure"), "(2+3)/2");
    screen.press_key(Key::Enter);
    assert_eq!(exposure.value_range().2, Some(2.5));
    let scale = section.get_by_role(Role::Button, "Scale: Not animated");
    assert!(
        section
            .get_by_role(Role::Button, "Keyed: Keyframe")
            .is_visible()
    );
    scale.click();
    section.get_by_role(Role::Button, "Scale: Keyframe").click();
    section.get_by_role(Role::Button, "Scale: Not animated");
    evidence(&screen, "creative-lime-compact-numeric");
    chapter(&screen, "buttons");
    let section = screen.get_by_test_id("creative-section-1");
    let apply = section.get_by_role(Role::Button, "Apply");
    apply.hover();
    let hover = screen.screenshot().hash();
    evidence(&screen, "creative-button-hover");
    screen.mouse_down(MouseButton::Primary);
    assert_ne!(screen.screenshot().hash(), hover);
    evidence(&screen, "creative-button-pressed");
    screen.mouse_up(MouseButton::Primary);
    apply.focus();
    assert!(apply.is_focused());
    assert!(section.get_by_role(Role::Button, "Disabled").is_disabled());
    evidence(&screen, "creative-button-focus-disabled");
    clean(&screen);
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
fn creative_typed_node_parallel_rewire_parameters_and_mute() {
    let screen = mount();
    chapter(&screen, "nodes");
    let canvas = screen.get_by_test_id("creative-node-editor");
    assert!(
        canvas
            .value()
            .expect("model summary")
            .starts_with("8 items")
    );
    assert!(screen.get_by_text("8 nodes · 9 links · 85%").exists());
    let input = canvas.get_by_role(Role::Button, "Blur input Image");
    let from = local(&screen, "creative-node-editor", input);
    input.drag(from, Point::new(from.x - 55.0, from.y + 50.0));
    assert!(screen.get_by_text("8 nodes · 8 links · 85%").exists());
    canvas
        .get_by_role(Role::Button, "Render Layers output Alpha")
        .a11y(A11yAction::Activate);
    canvas
        .get_by_role(Role::Button, "Blur input Image")
        .a11y(A11yAction::Activate);
    assert!(screen.get_by_text("8 nodes · 8 links · 85%").exists());
    canvas
        .get_by_role(Role::Button, "Render Layers output Image")
        .a11y(A11yAction::Activate);
    canvas
        .get_by_role(Role::Button, "Blur input Image")
        .a11y(A11yAction::Activate);
    assert!(screen.get_by_text("8 nodes · 9 links · 85%").exists());
    let factor = canvas.get_by_role(Role::SpinButton, "Mix Factor");
    factor.a11y_with_value(A11yAction::SetValue, "0.75");
    assert_eq!(factor.value_range().2, Some(0.75));
    factor.a11y_with_value(A11yAction::SetValue, "NaN");
    assert_eq!(factor.value_range().2, Some(0.75));
    canvas
        .get_by_role(Role::Button, "Mix")
        .a11y(A11yAction::Select);
    let before = screen.screenshot().hash();
    canvas.press_key(Key::Char('m'));
    assert_ne!(before, screen.screenshot().hash());
    evidence(&screen, "creative-node-rewire-mute");
    let mix = canvas.get_by_role(Role::Button, "Mix");
    let initial = mix.rect().expect("node header");
    canvas.press_key(Key::ArrowRight);
    let moved = mix.rect().expect("moved node header");
    assert!(moved.x > initial.x);
    evidence(&screen, "creative-node-application-move-preview");
    canvas.press_key(Key::Delete);
    assert!(canvas.value().expect("summary").starts_with("7 items"));
    clean(&screen);
}

#[test]
fn creative_bezier_tangent_and_interpolation_change_the_graph() {
    let screen = mount();
    chapter(&screen, "animation");
    let canvas = screen.get_by_test_id("creative-graph");
    let handle = canvas.get_by_role(Role::Button, "Z Location key 40 outgoing tangent");
    let p = local(&screen, "creative-graph", handle);
    let before = handle.rect().expect("handle").x;
    handle.drag(p, Point::new(p.x + 30.0, p.y + 20.0));
    assert!((handle.rect().expect("updated handle").x - before - 30.0).abs() < 0.1);
    canvas.press_key(Key::ArrowRight);
    evidence(&screen, "creative-bezier-tangent");
    let section = screen.get_by_test_id("creative-section-15");
    section.get_by_role(Role::RadioButton, "Linear").click();
    assert!(
        section
            .get_by_role(Role::RadioButton, "Linear")
            .is_checked()
            == Some(true)
    );
    evidence(&screen, "creative-linear-interpolation");
    clean(&screen);
}

#[test]
fn creative_viewport_orbit_grid_shading_and_dialog_focus() {
    let screen = mount();
    chapter(&screen, "viewport");
    let canvas = screen.get_by_test_id("creative-viewport");
    let r = canvas.rect().expect("viewport");
    let before = screen.screenshot().hash();
    screen.mouse_move(r.x + 250.0, r.y + 180.0);
    screen.mouse_down(MouseButton::Middle);
    screen.mouse_move(r.x + 300.0, r.y + 200.0);
    screen.mouse_up(MouseButton::Middle);
    assert_ne!(before, screen.screenshot().hash());
    let section = screen.get_by_test_id("creative-section-19");
    section.get_by_role(Role::RadioButton, "Wire").click();
    section.get_by_role(Role::Button, "Grid").click();
    assert_eq!(
        section.get_by_role(Role::Button, "Grid").is_checked(),
        Some(false)
    );
    evidence(&screen, "creative-viewport-wire-no-grid");
    assert!(canvas.get_by_role(Role::Image, "Sample area").is_visible());
    evidence(&screen, "creative-viewport-image-region");
    chapter(&screen, "dialogs");
    let opener = screen
        .get_by_test_id("creative-export-dialog")
        .get_by_role(Role::Button, "Render Settings…");
    opener.focus();
    opener.click();
    let window = screen.get_by_role(Role::Group, "Render Settings window");
    let bounds = window.rect().expect("render window");
    assert!((bounds.x - 1481.0).abs() < 0.5, "{bounds:?}");
    assert!((bounds.y - 120.0).abs() < 0.5, "{bounds:?}");
    assert!((bounds.width - 340.0).abs() < 0.5, "{bounds:?}");
    assert!((bounds.height - 285.0).abs() < 5.0, "{bounds:?}");
    assert!(screen.get_by_role(Role::Dialog, "Render Settings").exists());
    evidence(&screen, "creative-render-settings-format");
    let x = window.get_by_role(Role::SpinButton, "X");
    x.a11y_with_value(A11yAction::SetValue, "1280");
    assert_eq!(x.value_range().2, Some(1280.0));
    window.get_by_role(Role::Tab, "Encoding").click();
    assert!(window.get_by_role(Role::Tab, "Encoding").is_selected());
    let container = window.get_by_role(Role::ComboBox, "Render container");
    container.click();
    screen.get_by_role(Role::Option, "MP4").click();
    assert_eq!(container.value().as_deref(), Some("MP4"));
    let audio = window.get_by_role(Role::CheckBox, "Embed 48 kHz PCM");
    audio.click();
    assert_eq!(audio.is_checked(), Some(false));
    evidence(&screen, "creative-render-settings-encoding");
    window.get_by_role(Role::Tab, "Format").click();
    assert_eq!(x.value_range().2, Some(1280.0));
    screen.get_by_role(Role::Button, "Minimize window").click();
    assert!(!x.is_visible());
    assert!((window.rect().expect("collapsed").y - 120.0).abs() < 0.5);
    screen.get_by_role(Role::Button, "Minimize window").click();
    assert!(x.is_visible());
    let title = window.get_by_role(Role::Button, "Render Settings title bar");
    title.drag(Point::new(80.0, 20.0), Point::new(55.0, 40.0));
    let moved = window.rect().expect("moved window");
    assert!((moved.x - bounds.x + 25.0).abs() < 0.5);
    assert!((moved.y - bounds.y - 20.0).abs() < 0.5);
    window.get_by_role(Role::Button, "Cancel").click();
    assert!(screen.get_by_text("Render settings cancelled").exists());
    assert!(opener.is_focused());
    opener.click();
    screen
        .get_by_role(Role::Group, "Render Settings window")
        .get_by_role(Role::Button, "Render")
        .click();
    assert!(screen.get_by_text("Render requested (demo)").exists());
    assert!(opener.is_focused());
    opener.click();
    screen.press_key(Key::Escape);
    assert!(opener.is_focused());
    let close_opener = screen.get_by_role(Role::Button, "Close Project…");
    for (action, status) in [
        ("Don't Save", "Closed without saving (demo)"),
        ("Cancel", "Close cancelled"),
        ("Save", "Saved and closed (demo)"),
    ] {
        close_opener.click();
        let modal = screen.get_by_role(Role::Group, "Close Project window");
        assert!(modal.get_by_role(Role::Button, "Save").is_focused());
        let modal_bounds = modal.rect().expect("close modal");
        assert!((modal_bounds.width - 440.0).abs() < 0.5);
        assert!(
            (modal_bounds.height - 199.4375).abs() < 4.0,
            "{modal_bounds:?}"
        );
        assert!((modal_bounds.x - 710.5).abs() < 0.5);
        assert!((modal_bounds.y - 384.78125).abs() < 2.0);
        assert!(
            modal
                .get_by_role(
                    Role::Heading,
                    "Save changes to “Shot_010_Comp_v012” before closing?"
                )
                .exists()
        );
        evidence(&screen, "creative-close-project-confirmation");
        modal.get_by_role(Role::Button, action).click();
        assert!(screen.get_by_text(status).exists());
        assert!(close_opener.is_focused());
        assert!(
            screen
                .query_by_role(Role::Dialog, "Close Project")
                .is_none()
        );
    }
    close_opener.click();
    assert!(
        screen
            .get_by_role(Role::Group, "Close Project window")
            .get_by_role(Role::Button, "Save")
            .is_focused()
    );
    screen.press_key(Key::Enter);
    assert!(screen.get_by_text("Saved and closed (demo)").exists());
    assert!(close_opener.is_focused());
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

#[test]
fn creative_animation_projection_keeps_graph_and_dope_edits_shared() {
    let screen = mount();
    chapter(&screen, "animation");
    let dope = screen.get_by_test_id("creative-dopesheet");
    let graph = screen.get_by_test_id("creative-graph");
    let point = graph.get_by_role(Role::Button, "Z Location key 40");
    let before = point.rect().expect("graph point");
    let key_count = |canvas: Element| {
        canvas
            .value()
            .and_then(|value| {
                value
                    .split_whitespace()
                    .next()
                    .and_then(|n| n.parse::<usize>().ok())
            })
            .expect("public item count")
    };
    let graph_count = key_count(graph);
    let dope_count = key_count(dope);
    let key = dope.get_by_role(Role::Button, "Z Location · frame 40");
    key.a11y(A11yAction::Select);
    dope.press_key(Key::ArrowRight);
    let after = point.rect().expect("moved graph point");
    assert!(after.x > before.x);
    assert!(
        screen
            .within(dope)
            .query_by_role(Role::Button, "Z Location · frame 40")
            .is_none()
    );
    point.a11y(A11yAction::Select);
    graph.press_key(Key::Delete);
    assert_eq!(key_count(graph), graph_count - 1);
    assert!(key_count(dope) < dope_count);
    assert!(
        screen
            .within(graph)
            .query_by_role(Role::Button, "Z Location key 40")
            .is_none()
    );
    assert_eq!(
        screen
            .within(dope)
            .all_by_role(Role::Button, "Z Location · frame 40")
            .len(),
        0
    );
    evidence(&screen, "creative-shared-animation");
    clean(&screen);
}
#[test]
fn creative_thumbnail_modifier_selection_and_pie_hold_release_update_viewport() {
    use kivact::view::KeyCode;
    let screen = mount();
    chapter(&screen, "media");
    let modes = screen.get_by_test_id("creative-media-mode");
    modes.get_by_role(Role::RadioButton, "⊞").click();
    let first = screen.get_by_role(Role::ListItem, "A001_C003_0912");
    let second = screen.get_by_role(Role::ListItem, "A001_C007_0912");
    let third = screen.get_by_role(Role::ListItem, "A002_C001_0913");
    first.click();
    screen.key_down(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    second.click();
    screen.key_up(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers::default(),
    );
    assert!(first.is_selected() && second.is_selected());
    screen.key_down(
        Key::Unidentified,
        KeyCode::ShiftLeft,
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );
    third.click();
    screen.key_up(Key::Unidentified, KeyCode::ShiftLeft, Modifiers::default());
    assert!(first.is_selected() && second.is_selected() && third.is_selected());
    screen.key_down(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    second.click();
    screen.key_up(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers::default(),
    );
    assert!(!second.is_selected());
    evidence(&screen, "creative-thumbnail-range");
    chapter(&screen, "menus");
    let zone = screen.get_by_test_id("creative-pie-zone");
    zone.focus();
    zone.hover();
    let center = zone.rect().expect("pie zone").center();
    screen.key_down(Key::Char('z'), KeyCode::KeyZ, Modifiers::default());
    assert!(screen.get_by_role(Role::Menu, "Shading pie").is_visible());
    screen.mouse_move(center.x - 104.0, center.y);
    evidence(&screen, "creative-pie-held");
    screen.key_up(Key::Char('z'), KeyCode::KeyZ, Modifiers::default());
    assert!(screen.query_by_role(Role::Menu, "Shading pie").is_none());
    chapter(&screen, "viewport");
    assert!(screen.get_by_role(Role::RadioButton, "Wire").is_checked() == Some(true));
    chapter(&screen, "menus");
    screen
        .get_by_test_id("creative-section-operator-search")
        .click();
    let search = screen.get_by_role(Role::Dialog, "Operator Search");
    fill(
        search.get_by_role(Role::TextBox, "Search operators"),
        "Node Editor",
    );
    assert_eq!(
        screen
            .within(search)
            .all_by_role(Role::Button, "Node Editor")
            .len(),
        1
    );
    assert!(
        screen
            .within(search)
            .query_by_role(Role::Button, "Buttons")
            .is_none()
    );
    evidence(&screen, "creative-section-search");
    search.get_by_role(Role::Button, "Node Editor").click();
    assert!(
        screen
            .query_by_role(Role::Dialog, "Operator Search")
            .is_none()
    );
    assert!(screen.get_by_text("Operator · Node Editor").is_visible());
    assert!(screen.get_by_test_id("creative-node-editor").is_visible());
    screen.press_key(Key::F(3));
    assert!(
        screen
            .get_by_role(Role::Dialog, "Operator Search")
            .is_visible()
    );
    screen.press_key(Key::Escape);
    screen
        .get_by_test_id("creative-top-operator-search")
        .click();
    assert!(
        screen
            .get_by_role(Role::Dialog, "Operator Search")
            .is_visible()
    );
    screen.press_key(Key::Escape);
    clean(&screen);
}
#[test]
fn creative_job_severity_filter_and_removal_stop_observable_work() {
    let screen = mount();
    chapter(&screen, "jobs");
    let section = screen.get_by_test_id("creative-section-22");
    let log = section.get_by_role(Role::TextBox, "Job log");
    assert!(log.value().is_some_and(|value| !value.is_empty()));
    section.get_by_test_id("creative-log-filter-0").click();
    assert_eq!(log.value(), Some(String::new()));
    section
        .get_by_test_id("creative-log-report-Warning")
        .click();
    assert!(log.value().is_some_and(|value| value.contains("WARNING")));
    assert!(
        screen.get_by_role(Role::Alert, "Catalog report").exists(),
        "a warning report is an assertive toast"
    );
    screen.advance(Duration::from_secs(5));
    assert!(
        screen
            .query_by_role(Role::Alert, "Catalog report")
            .is_none()
    );
    section.get_by_test_id("creative-log-filter-2").click();
    assert_eq!(log.value(), Some(String::new()));
    section.get_by_test_id("creative-log-filter-2").click();
    section.get_by_role(Role::Button, "Remove job 2").click();
    assert!(
        screen
            .within(section)
            .query_by_role(Role::ProgressBar, "Job 2 progress")
            .is_none()
    );
    section.get_by_test_id("creative-start-job").click();
    screen.advance(Duration::from_millis(500));
    let remaining = section.get_by_role(Role::ProgressBar, "Job 3 progress");
    assert!(remaining.value_range().2.unwrap_or(0.0) > 0.0);
    section.get_by_role(Role::Button, "Stop").click();
    let stopped = remaining.value_range().2;
    screen.advance(Duration::from_millis(500));
    assert_eq!(remaining.value_range().2, stopped);
    evidence(&screen, "creative-filtered-job-log");
    clean(&screen);
}

#[path = "arrangement.rs"]
mod arrangement;
#[test]
fn catalog_outliner_applies_rename_restriction_and_reparent_requests() {
    let screen = mount();
    chapter(&screen, "outliner");
    let tree = screen.get_by_test_id("creative-scene-outliner");
    let camera = tree.get_by_role(Role::TreeItem, "Camera");
    camera.click();
    screen.press_key(Key::F(2));
    fill(
        screen.get_by_role(Role::TextBox, "Rename tree item"),
        "Catalog Camera",
    );
    screen.press_key(Key::Enter);
    let camera = tree.get_by_role(Role::TreeItem, "Catalog Camera");
    assert!(camera.exists());
    camera
        .get_by_role(Role::Button, "Tree visibility")
        .a11y(A11yAction::Toggle);
    assert_eq!(
        camera
            .get_by_role(Role::Button, "Tree visibility")
            .is_checked(),
        Some(false)
    );
    let from = camera.rect().expect("camera row");
    let over = tree
        .get_by_role(Role::TreeItem, "Environment")
        .rect()
        .expect("environment row");
    screen.advance(Duration::from_secs(1));
    camera.drag(
        Point::new(100.0, from.height / 2.0),
        Point::new(100.0, over.center().y - from.y),
    );
    assert_eq!(
        tree.get_by_role(Role::TreeItem, "Catalog Camera").level(),
        Some(3)
    );
    evidence(&screen, "creative-outliner-applied-name-restriction-drop");
    tree.get_by_role(Role::TreeItem, "Environment").click();
    screen.press_key(Key::ArrowLeft);
    assert!(
        screen
            .query_by_role(Role::TreeItem, "Catalog Camera")
            .is_none()
    );
    evidence(&screen, "creative-outliner-application-reducers");
    clean(&screen);
}

#[test]
fn catalog_layer_table_applies_reorder_requests() {
    let screen = mount();
    chapter(&screen, "layers");
    let grade = screen.get_by_role(Role::Row, "Grade [Adjustment]");
    let before = grade.rect().expect("grade row");
    let title = screen
        .get_by_role(Role::Row, "Title_Main")
        .rect()
        .expect("title row");
    assert!(before.y < title.y);
    grade.drag(
        Point::new(300.0, before.height / 2.0),
        Point::new(300.0, before.height / 2.0 + 35.0),
    );
    assert!(
        screen
            .get_by_role(Role::Row, "Grade [Adjustment]")
            .rect()
            .unwrap()
            .y
            > screen
                .get_by_role(Role::Row, "Title_Main")
                .rect()
                .unwrap()
                .y
    );
    evidence(&screen, "creative-layer-application-reorder");
    clean(&screen);
}
