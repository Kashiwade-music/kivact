//! Role: Input, scroll and portable platform contracts on real windows.
use kivact::primitive::NodeExt;

use crate::support::mount;
use kivact::platform;
use kivact::prelude::*;
use kivact::style::{Align, Color};
use kivact::task::spawn;
use kivact::view::{KeyCode, LiveRegion, Modifiers, Orientation, SortDirection, callback};

#[test]
fn a_press_handler_reads_the_modifiers_held_at_the_press() {
    let (screen, seen) = mount(|| {
        let seen = signal(None::<bool>);
        let view =
            button("Open").on_press(move || seen.set(Some(kivact::view::current_modifiers().ctrl)));
        (seen, view)
    });
    let ctrl = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    screen.key_down(Key::Unidentified, KeyCode::ControlLeft, ctrl);
    screen.get_by_role(Role::Button, "Open").click();
    assert_eq!(
        seen.get_untracked(),
        Some(true),
        "Ctrl is held at the press"
    );
    screen.key_up(
        Key::Unidentified,
        KeyCode::ControlLeft,
        Modifiers::default(),
    );
    screen.get_by_role(Role::Button, "Open").click();
    assert_eq!(seen.get_untracked(), Some(false), "Ctrl was released");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn input_alignment_keeps_pixels_pointer_selection_and_ime_in_the_same_place() {
    let (screen, value) = mount(|| {
        let value = signal("AB".to_owned());
        let field = |id, alignment| {
            kivact::control::node()
                .name(id)
                .part(kivact::interaction::use_text_edit(value).control())
                .test_id(id)
                .style(
                    style()
                        .width(240)
                        .height(42)
                        .padding_x(8)
                        .font_size(20)
                        .text_align(alignment)
                        .color(Color::BLACK)
                        .background(Color::WHITE),
                )
        };
        (
            value,
            column((
                field("start", kivact::style::TextAlign::Start),
                field("center", kivact::style::TextAlign::Center),
                field("end", kivact::style::TextAlign::End),
            ))
            .style(
                style()
                    .padding(20)
                    .gap(20)
                    .align_items(Align::Start)
                    .background(Color::WHITE),
            ),
        )
    });
    let frame = screen.screenshot();
    let first_ink = |id| {
        let r = screen.get_by_test_id(id).rect().unwrap();
        let field = frame.crop(r.x as u32, r.y as u32, 240, 42);
        (0..240)
            .find(|&x| {
                (0..42).any(|y| {
                    field.pixel(x, y).is_some_and(|p| {
                        [0, 8, 16].into_iter().all(|shift| (p >> shift) & 255 < 100)
                    })
                })
            })
            .expect("painted input text") as f32
    };
    assert!(first_ink("center") > first_ink("start") + 80.0);
    assert!(first_ink("end") > first_ink("center") + 80.0);
    for id in ["center", "end"] {
        let field = screen.get_by_test_id(id);
        field.focus();
        field.press_key(Key::Home);
        let start = screen.ime_text().unwrap();
        let first = *start.char_rects.last().unwrap();
        assert!((first.x - start.area.x - first_ink(id) + 8.0).abs() <= 4.0);
        field.press_key(Key::End);
        let last = *screen.ime_text().unwrap().char_rects.last().unwrap();
        screen.click_at(first.x, first.y + first.height / 2.0);
        assert_eq!(screen.ime_text().unwrap().selection, 0..0);
        screen.click_at(last.x, last.y + last.height / 2.0);
        assert_eq!(screen.ime_text().unwrap().selection, 2..2);
        field.reconvert(0..1);
        assert_eq!(screen.ime_text().unwrap().selection, 0..1);
        field.compose("漢");
        let composed = screen.ime_text().unwrap();
        assert_eq!(composed.text, "漢B");
        assert_eq!(composed.composition, Some(0..1));
        assert!(
            composed
                .char_rects
                .iter()
                .all(|r| r.x >= composed.area.x - 2.0
                    && r.x <= composed.area.x + composed.area.width + 2.0)
        );
        field.commit("AB");
        value.set("AB".to_owned());
        screen.run_until_idle();
    }
    value.set("A".repeat(100));
    screen.run_until_idle();
    let center = screen.get_by_test_id("center");
    center.focus();
    center.press_key(Key::End);
    let end = screen.ime_text().unwrap();
    let caret = end.char_rects.last().unwrap();
    assert!(caret.x >= end.area.x && caret.x <= end.area.x + end.area.width);
    center.press_key(Key::Home);
    let start = screen.ime_text().unwrap();
    assert!((start.char_rects.last().unwrap().x - start.area.x).abs() <= 2.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn password_updates_and_ime_keep_published_text_protected() {
    let (screen, (value, password)) = mount(|| {
        let value = signal("DummyInitial_27".to_owned());
        let password = signal(true);
        (
            (value, password),
            kivact::control::node().name("Protected fixture").part(
                kivact::interaction::use_text_edit(value)
                    .password(password)
                    .control(),
            ),
        )
    });
    let input = screen.get_by_role(Role::TextBox, "Protected fixture");
    input.focus();
    let assert_protected = || {
        assert!(input.password());
        assert_eq!(input.value(), None);
        let ime = screen.ime_text().expect("focused protected input");
        assert_eq!(ime.text, "");
        assert_eq!(ime.selection, 0..0);
        assert!(ime.composition.is_none());
        assert!(!screen.accessibility_tree().contains("Dummy"));
    };
    assert_protected();
    value.set("DummyUpdate_42".to_owned());
    screen.run_until_idle();
    assert_protected();
    input.press_key(Key::End);
    input.compose("🦀b");
    assert_protected();
    input.commit("🦀b");
    assert_eq!(value.get_untracked(), "DummyUpdate_42🦀b");
    assert_protected();
    password.set(false);
    screen.run_until_idle();
    assert_eq!(input.value(), Some("DummyUpdate_42🦀b".to_owned()));
    assert_eq!(
        screen.ime_text().expect("revealed input").text,
        value.get_untracked()
    );
    password.set(true);
    screen.run_until_idle();
    assert_protected();
    input.type_text("z");
    assert_protected();
    assert!(value.get_untracked().ends_with("🦀bz"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn advanced_roles_properties_active_descendant_and_fullscreen_are_published() {
    let (screen, (value, fullscreen, set_fullscreen)) = mount(|| {
        let value = signal(4.0);
        let active = node_ref();
        let set_fullscreen = callback(move |on| {
            spawn(platform::set_fullscreen(on));
        });
        let fullscreen = memo(move || platform::fullscreen());
        (
            (value, fullscreen, set_fullscreen),
            column((
                node()
                    .role(Role::SpinButton)
                    .name("Number")
                    .value_min(0.0)
                    .value_max(10.0)
                    .value_now(value)
                    .orientation(Orientation::Vertical),
                node().role(Role::Meter).name("Meter").value_now(0.5),
                node()
                    .role(Role::MenuBar)
                    .name("Bar")
                    .child(node().role(Role::MenuItemCheckBox).name("Check")),
                node().role(Role::MenuItemRadio).name("Radio"),
                node().role(Role::Separator).name("Separator"),
                node()
                    .role(Role::Grid)
                    .name("Grid")
                    .active_descendant(Some(active))
                    .child(
                        node()
                            .node_ref(active)
                            .role(Role::GridCell)
                            .name("Cell")
                            .sort(SortDirection::Ascending),
                    ),
                node()
                    .role(Role::Status)
                    .name("Status")
                    .live_region(LiveRegion::Assertive),
            )),
        )
    });
    let input = screen.get_by_role(Role::SpinButton, "Number");
    assert_eq!(input.value_range(), (Some(0.0), Some(10.0), Some(4.0)));
    assert_eq!(input.orientation(), Orientation::Vertical);
    value.set(7.0);
    screen.run_until_idle();
    assert_eq!(input.value_range().2, Some(7.0));
    let grid = screen.get_by_role(Role::Grid, "Grid");
    let cell = screen.get_by_role(Role::GridCell, "Cell");
    assert!(grid.has_active_descendant(&cell));
    assert_eq!(cell.sort(), SortDirection::Ascending);
    assert_eq!(
        screen.get_by_role(Role::Status, "Status").live_region(),
        LiveRegion::Assertive
    );
    assert!(!fullscreen.get_untracked());
    set_fullscreen.call(true);
    screen.run_until_idle();
    assert!(fullscreen.get_untracked());
    set_fullscreen.call(false);
    screen.run_until_idle();
    assert!(!fullscreen.get_untracked());
    assert!(screen.errors().is_empty());
}

#[test]
fn a_reactive_active_descendant_moves_in_the_accessibility_tree() {
    let (screen, which) = mount(|| {
        let which = signal(Some(0usize));
        let refs = [node_ref(), node_ref()];
        (
            which,
            node()
                .role(Role::ListBox)
                .name("Results")
                .focusable(true)
                .active_descendant(move || which.get().map(|i| refs[i]))
                .child((
                    node()
                        .node_ref(refs[0])
                        .role(Role::Option)
                        .name("First")
                        .child(text("First")),
                    node()
                        .node_ref(refs[1])
                        .role(Role::Option)
                        .name("Second")
                        .child(text("Second")),
                )),
        )
    });
    let list = screen.get_by_role(Role::ListBox, "Results");
    let first = screen.get_by_role(Role::Option, "First");
    let second = screen.get_by_role(Role::Option, "Second");
    assert!(list.has_active_descendant(&first));
    which.set(Some(1));
    screen.run_until_idle();
    assert!(list.has_active_descendant(&second));
    assert!(!list.has_active_descendant(&first));
    which.set(None);
    screen.run_until_idle();
    assert!(!list.has_active_descendant(&first));
    assert!(!list.has_active_descendant(&second));
    which.set(Some(0));
    screen.run_until_idle();
    assert!(list.has_active_descendant(&first));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
