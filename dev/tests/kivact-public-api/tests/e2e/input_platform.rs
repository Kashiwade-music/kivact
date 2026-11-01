//! Role: Input, scroll and portable platform contracts on real windows.
use kivact::primitive::NodeExt;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use crate::support::mount;
use kivact::draw::Point;
use kivact::draw::Transform;
use kivact::platform;
use kivact::prelude::*;
use kivact::style::{Align, Color};
use kivact::style::{Cursor, ScrollbarWidth};
use kivact::task::spawn;
use kivact::test::render;
use kivact::view::KeyEvent;
use kivact::view::{
    FilePhase, KeyCode, LiveRegion, Modifiers, Orientation, ScrollBehavior, SortDirection, callback,
};

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
fn password_pixels_mask_the_initial_value_and_follow_reveal_changes() {
    let (screen, password) = mount(|| {
        let password = signal(true);
        let field_style = style()
            .width(240)
            .height(36)
            .font_size(20)
            .color(Color::BLACK)
            .background(Color::WHITE);
        (
            password,
            column((
                kivact::control::node()
                    .name("Protected fixture")
                    .part(
                        kivact::interaction::use_text_edit(signal("Secret123".to_owned()))
                            .password(password)
                            .control(),
                    )
                    .test_id("protected")
                    .style(field_style.clone()),
                kivact::control::node()
                    .name("Bullet reference")
                    .part(
                        kivact::interaction::use_text_edit(signal("•••••••••".to_owned()))
                            .control(),
                    )
                    .test_id("bullets")
                    .style(field_style.clone()),
                kivact::control::node()
                    .name("Visible reference")
                    .part(
                        kivact::interaction::use_text_edit(signal("Secret123".to_owned()))
                            .control(),
                    )
                    .test_id("visible")
                    .style(field_style),
            ))
            .style(
                style()
                    .padding(20)
                    .gap(24)
                    .align_items(Align::Start)
                    .background(Color::WHITE),
            ),
        )
    });
    let pixels = |id: &str| {
        let rect = screen.get_by_test_id(id).rect().expect("field laid out");
        screen
            .screenshot()
            .crop(rect.x as u32, rect.y as u32, 240, 36)
    };
    let assert_same_pixels = |actual: kivact::test::Screenshot,
                              expected: kivact::test::Screenshot| {
        // Glyph atlas sampling at different window positions may round a
        // coverage channel by one. Plain text versus bullets differs by 255.
        let mut difference = 0u64;
        for (a, b) in actual.pixels.iter().zip(&expected.pixels) {
            for shift in [0, 8, 16] {
                let delta = ((a >> shift) & 255).abs_diff((b >> shift) & 255);
                assert!(
                    delta <= 2,
                    "different displayed glyphs, channel delta {delta}"
                );
                difference += u64::from(delta);
            }
        }
        assert!(
            difference <= 240,
            "different displayed glyphs, total delta {difference}"
        );
    };
    let protected = screen.get_by_test_id("protected");
    assert!(protected.password());
    assert_eq!(protected.value(), None);
    assert_same_pixels(pixels("protected"), pixels("bullets"));
    assert_ne!(pixels("protected").hash(), pixels("visible").hash());
    password.set(false);
    screen.run_until_idle();
    assert_same_pixels(pixels("protected"), pixels("visible"));
    password.set(true);
    screen.run_until_idle();
    assert_same_pixels(pixels("protected"), pixels("bullets"));
    assert!(protected.password());
    assert_eq!(protected.value(), None);
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
fn releases_keep_physical_codes_and_cannot_activate() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let clicks = Rc::new(std::cell::Cell::new(0));
    let activated = clicks.clone();
    let (down, up) = (events.clone(), events.clone());
    let screen = render(move || {
        node()
            .role(Role::Button)
            .focusable(true)
            .test_id("keys")
            .on_press(move || activated.set(activated.get() + 1))
            .style(style().size(80, 40))
            .on_key(move |event: KeyEvent| down.borrow_mut().push((false, event.code)))
            .on_key_up(move |event: KeyEvent| up.borrow_mut().push((true, event.code)))
    });
    screen.get_by_test_id("keys").focus();
    screen.key_down(Key::Char('5'), KeyCode::Digit5, Modifiers::default());
    screen.key_up(Key::Char('5'), KeyCode::Digit5, Modifiers::default());
    screen.key_down(Key::Char('5'), KeyCode::Numpad5, Modifiers::default());
    screen.key_up(Key::Char('5'), KeyCode::Numpad5, Modifiers::default());
    assert_eq!(
        &*events.borrow(),
        &[
            (false, KeyCode::Digit5),
            (true, KeyCode::Digit5),
            (false, KeyCode::Numpad5),
            (true, KeyCode::Numpad5)
        ]
    );
    screen.key_up(Key::Enter, KeyCode::Enter, Modifiers::default());
    assert_eq!(clicks.get(), 0);
    assert!(screen.errors().is_empty());
}

#[test]
fn cursor_stays_with_capture_and_exact_hit_follows_shapes_and_occlusion() {
    let (screen, drag_ref) = mount(|| {
        let node_ref = node_ref();
        (
            node_ref,
            column((
                node()
                    .node_ref(node_ref)
                    .test_id("drag")
                    .on_pointer(|_| ())
                    .style(style().size(80, 40).cursor(Cursor::Grab)),
                node()
                    .test_id("other")
                    .style(style().size(80, 40).cursor(Cursor::Crosshair)),
            )),
        )
    });
    screen.mouse_move(10.0, 10.0);
    assert_eq!(screen.cursor(), Cursor::Grab);
    screen.mouse_down(kivact::view::MouseButton::Primary);
    screen.mouse_move(10.0, 60.0);
    assert_eq!(screen.cursor(), Cursor::Grab);
    screen.mouse_up(kivact::view::MouseButton::Primary);
    assert_eq!(screen.cursor(), Cursor::Crosshair);
    assert!(drag_ref.hit_test(Point::new(10.0, 10.0)));
    assert!(!drag_ref.hit_test(Point::new(10.0, 60.0)));

    let (screen, target) = mount(|| {
        let target = node_ref();
        (
            target,
            node().style(style().size(140, 140)).child((
                node().node_ref(target).style(
                    style()
                        .size(80, 80)
                        .position(kivact::style::Position::Absolute)
                        .transform(Transform::new().rotate(45))
                        .clip_path(kivact::draw::Shape::circle(30)),
                ),
                node().style(
                    style()
                        .size(20, 20)
                        .position(kivact::style::Position::Absolute)
                        .left(30)
                        .top(30)
                        .z_index(2),
                ),
            )),
        )
    });
    assert!(!target.hit_test(Point::new(1.0, 1.0)));
    assert!(!target.hit_test(Point::new(35.0, 35.0)));
    assert!(target.hit_test(Point::new(60.0, 40.0)));
    assert!(screen.errors().is_empty());
}

#[test]
fn files_keep_positions_and_one_session_across_children() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let log = events.clone();
    let screen = render(move || {
        node()
            .style(style().size(120, 80).padding(10))
            .on_file(move |event: kivact::view::FileEvent| log.borrow_mut().push(event))
            .child((text("first"), text("second")))
    });
    let files = vec![PathBuf::from("one.png"), PathBuf::from("two.png")];
    screen.file_drag(FilePhase::Enter, files.clone(), Point::new(15.0, 15.0));
    screen.file_drag(FilePhase::Over, files.clone(), Point::new(15.0, 35.0));
    screen.file_drag(FilePhase::Drop, files.clone(), Point::new(20.0, 40.0));
    let events = events.borrow();
    assert_eq!(
        events
            .iter()
            .filter(|e| e.phase == FilePhase::Enter)
            .count(),
        1
    );
    let last = events.last().unwrap();
    assert_eq!(last.phase, FilePhase::Drop);
    assert_eq!(last.files, files);
    assert_eq!(last.window_pos, Point::new(20.0, 40.0));
    assert_eq!(last.pos, Point::new(20.0, 40.0));
    assert!(screen.errors().is_empty());
}

#[test]
fn password_length_readonly_selection_and_invalid_share_the_editor() {
    let (screen, (value, field, readonly, password)) = mount(|| {
        let value = signal(String::new());
        let field = node_ref();
        let readonly = signal(false);
        let password = signal(true);
        (
            (value, field, readonly, password),
            kivact::control::node()
                .name("Secret")
                .part(
                    kivact::interaction::use_text_edit(value)
                        .password(password)
                        .max_length(3)
                        .read_only(readonly)
                        .control(),
                )
                .node_ref(field)
                .invalid(true),
        )
    });
    let input = screen.get_by_role(Role::TextBox, "Secret");
    input.type_text("a🦀bc");
    assert_eq!(value.get_untracked(), "a🦀b");
    assert!(input.password());
    assert!(input.invalid());
    assert_eq!(input.value(), None);
    assert_eq!(screen.ime_text().expect("password IME purpose").text, "");
    input.reconvert(0..1);
    assert_eq!(
        screen.ime_text().expect("no surrounding text").selection,
        0..0
    );
    field.select_all();
    screen.run_until_idle();
    screen.set_clipboard("unchanged");
    input.press_key_with(
        Key::Char('c'),
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    assert_eq!(screen.clipboard().as_deref(), Some("unchanged"));
    password.set(false);
    screen.run_until_idle();
    assert_eq!(
        screen
            .ime_text()
            .expect("purpose follows reactive password")
            .text,
        value.get_untracked()
    );
    readonly.set(true);
    screen.run_until_idle();
    input.type_text("NO");
    input.press_key(Key::Backspace);
    input.press_key_with(
        Key::Char('x'),
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    assert_eq!(value.get_untracked(), "a🦀b");
    assert!(input.read_only());
    field.select_all();
    screen.run_until_idle();
    input.press_key_with(
        Key::Char('c'),
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    assert_eq!(screen.clipboard().as_deref(), Some("a🦀b"));
    readonly.set(false);
    screen.run_until_idle();
    input.type_text("🦀x");
    assert_eq!(value.get_untracked(), "🦀x");
    assert!(screen.errors().is_empty());
}

#[test]
fn password_ime_caret_uses_the_transformed_window_position() {
    let (screen, password) = mount(|| {
        let password = signal(false);
        (
            password,
            kivact::control::node()
                .name("Moved secret")
                .part(
                    kivact::interaction::use_text_edit(kivact::signal(String::new()))
                        .password(password)
                        .control(),
                )
                .style(style().translate(80, 40).scale(1.5)),
        )
    });
    screen.get_by_role(Role::TextBox, "Moved secret").focus();
    let normal = screen.ime_text().unwrap().char_rects[0];
    password.set(true);
    screen.run_until_idle();
    assert_eq!(screen.ime_text().unwrap().char_rects[0], normal);
}

#[test]
fn readonly_switch_commits_preedit_and_releases_the_ime() {
    let (screen, (value, readonly, keys)) = mount(|| {
        let value = signal(String::from("x"));
        let readonly = signal(false);
        let keys = signal(0usize);
        (
            (value, readonly, keys),
            node()
                .on_key(move |_| keys.set(keys.get_untracked() + 1))
                .child(
                    kivact::control::node().name("Changing").part(
                        kivact::interaction::use_text_edit(value)
                            .read_only(readonly)
                            .max_length(4usize)
                            .control(),
                    ),
                ),
        )
    });
    let input = screen.get_by_role(Role::TextBox, "Changing");
    input.press_key(Key::End);
    keys.set(0);
    input.compose("abcde");
    readonly.set(true);
    screen.run_until_idle();
    assert_eq!(value.get_untracked(), "xabc");
    assert!(screen.ime_text().is_none());
    input.press_key(Key::Backspace);
    assert_eq!(value.get_untracked(), "xabc");
    assert_eq!(keys.get_untracked(), 1);
    readonly.set(false);
    screen.run_until_idle();
    input.press_key(Key::Backspace);
    input.type_text("z");
    assert_eq!(value.get_untracked(), "xabz");
}

#[test]
fn scrollbar_state_style_changes_both_pixels_and_drag_hit() {
    let (screen, offset) = mount(|| {
        let node_ref = node_ref();
        let offset = node_ref.scroll_offset();
        (
            offset,
            node()
                .node_ref(node_ref)
                .style(
                    style()
                        .size(100, 100)
                        .overflow(kivact::style::Overflow::Scroll)
                        .scrollbar_width(6)
                        .scrollbar_color(rgb(0x0000ff), rgb(0))
                        .hover(
                            style()
                                .scrollbar_width(20)
                                .scrollbar_color(rgb(0xff0000), rgb(0)),
                        ),
                )
                .child(node().style(style().size(90, 500).shrink(0.0))),
        )
    });
    assert_eq!(screen.screenshot().pixel(96, 8), Some(0x0000ff));
    screen.mouse_move(85.0, 8.0);
    assert_eq!(screen.screenshot().pixel(85, 8), Some(0xff0000));
    screen.mouse_down(kivact::view::MouseButton::Primary);
    screen.mouse_move(85.0, 60.0);
    screen.mouse_up(kivact::view::MouseButton::Primary);
    assert!(offset.get_untracked().y > 200.0);
    screen.mouse_move(85.0, 60.0);
    screen.mouse_down(kivact::view::MouseButton::Primary);
    let before = offset.get_untracked();
    screen.set_window_focused(false);
    screen.set_window_focused(true);
    screen.mouse_move(85.0, 90.0);
    assert_eq!(
        offset.get_untracked(),
        before,
        "capture cancellation ends scrollbar drag"
    );
}

#[test]
fn scroll_offsets_smooth_motion_and_overlay_thumb_drag_are_observable() {
    let (screen, (node_ref, offset, hidden)) = mount(|| {
        let node_ref = node_ref();
        let offset = node_ref.scroll_offset();
        let hidden = signal(false);
        (
            (node_ref, offset, hidden),
            node()
                .node_ref(node_ref)
                .test_id("scroll")
                .style(move || {
                    style()
                        .size(100, 100)
                        .overflow(kivact::style::Overflow::Scroll)
                        .scrollbar_width(if hidden.get() {
                            ScrollbarWidth::None
                        } else {
                            ScrollbarWidth::Px(8.0)
                        })
                        .scrollbar_color(rgb(0xff0000), rgb(0x000000))
                })
                .child(node().style(style().size(90, 500).shrink(0.0))),
        )
    });
    node_ref.scroll_to(Point::new(0.0, 50.0), ScrollBehavior::Instant);
    screen.run_until_idle();
    assert_eq!(offset.get_untracked().y, 50.0);
    node_ref.scroll_by(Point::new(0.0, 100.0), ScrollBehavior::Smooth);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(100));
    let mid = offset.get_untracked().y;
    assert!(mid > 50.0 && mid < 150.0, "{mid}");
    screen.advance(Duration::from_millis(300));
    assert_eq!(offset.get_untracked().y, 150.0);
    node_ref.scroll_to(Point::default(), ScrollBehavior::Instant);
    screen.run_until_idle();
    assert_eq!(screen.screenshot().pixel(96, 8), Some(0xff0000));
    screen.mouse_move(96.0, 8.0);
    screen.mouse_down(kivact::view::MouseButton::Primary);
    screen.mouse_move(96.0, 60.0);
    screen.mouse_up(kivact::view::MouseButton::Primary);
    assert!(offset.get_untracked().y > 200.0);
    hidden.set(true);
    screen.run_until_idle();
    assert_ne!(screen.screenshot().pixel(96, 8), Some(0xff0000));
    screen.set_reduced_motion(true);
    node_ref.scroll_to(Point::new(0.0, 30.0), ScrollBehavior::Smooth);
    screen.run_until_idle();
    assert_eq!(offset.get_untracked().y, 30.0);
    assert!(screen.errors().is_empty());
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

#[test]
fn an_active_descendant_in_a_controlled_node_is_published_and_one_outside_is_not() {
    let (screen, ()) = mount(|| {
        let list = node_ref();
        let first = node_ref();
        let other = node_ref();
        let option = |r: NodeRef, name: &'static str| {
            node()
                .node_ref(r)
                .role(Role::Option)
                .name(name)
                .child(text(name))
        };
        (
            (),
            column((
                kivact::control::node()
                    .name("Controlling")
                    .part(
                        kivact::interaction::use_text_edit(kivact::signal(String::new())).control(),
                    )
                    .controls(list)
                    .active_descendant(Some(first)),
                kivact::control::node()
                    .name("Uncontrolling")
                    .part(
                        kivact::interaction::use_text_edit(kivact::signal(String::new())).control(),
                    )
                    .active_descendant(Some(first)),
                kivact::control::node()
                    .name("Elsewhere")
                    .part(
                        kivact::interaction::use_text_edit(kivact::signal(String::new())).control(),
                    )
                    .controls(list)
                    .active_descendant(Some(other)),
                node()
                    .node_ref(list)
                    .role(Role::ListBox)
                    .name("List")
                    .child(option(first, "First")),
                column(option(other, "Other"))
                    .role(Role::ListBox)
                    .name("Aside"),
            )),
        )
    });
    let first = screen.get_by_role(Role::Option, "First");
    let other = screen.get_by_role(Role::Option, "Other");
    let field = |name: &str| screen.get_by_role(Role::TextBox, name);
    assert!(
        field("Controlling").has_active_descendant(&first),
        "an option inside the node it controls"
    );
    assert!(
        !field("Uncontrolling").has_active_descendant(&first),
        "an option of a list it does not control"
    );
    assert!(
        !field("Elsewhere").has_active_descendant(&other),
        "an option outside the node it controls"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
