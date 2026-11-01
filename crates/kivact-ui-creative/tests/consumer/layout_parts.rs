//! Role: Tests for the split in the kit's theme, property rows, windows and panels.
//!
//! The standard split, property rows and state buttons, windows an application opens
//! and closes, and panels with a header of their own.

use super::*;
use kivact::draw::Rect;
use kivact::i18n::Messages;
use kivact::primitive::NodeExt;
use kivact::test::Element;
use kivact::view::{Orientation, node_ref};
use ui::PropertyState::{Animated, Driven, Keyed, Plain};

fn width(element: &Element<'_>) -> f32 {
    element.rect().expect("laid out").width
}

fn height(element: &Element<'_>) -> f32 {
    element.rect().expect("laid out").height
}

/// A standard split of two empty panes in a box of `length` by 100 logical
/// pixels (stacked: 100 by `length`), drawn with the kit's theme.
fn mount_split(axis: Orientation, sizes: Vec<f32>, length: f32) -> (Screen, Signal<Vec<f32>>) {
    kivact::test::render_with_handle(crate::options(), move || {
        let sizes = signal(sizes);
        (
            sizes,
            node()
                .style(if axis == Orientation::Vertical {
                    style().width(100).height(length)
                } else {
                    style().width(length).height(100)
                })
                .child(
                    kivact::ui::split(
                        (
                            node().test_id("first").style(style().grow(1)),
                            node().test_id("second").style(style().grow(1)),
                        ),
                        sizes,
                    )
                    .orientation(axis),
                ),
        )
    })
}

#[test]
fn the_standard_split_draws_the_docks_splitter_and_moves_along_its_axis() {
    let (screen, sizes) = mount_split(Orientation::Horizontal, vec![0.25, 0.75], 406.0);
    // 400 pixels without the 6 pixel splitter, as thick as the dock's.
    let width_of = |id: &str| width(&screen.get_by_test_id(id));
    assert!(near(width_of("first"), 100.0, 0.6) && near(width_of("second"), 300.0, 0.6));
    let splitter = screen.get_by_role(Role::Splitter, "Resize");
    assert!(near(width(&splitter), 6.0, 0.6));
    assert_eq!(splitter.orientation(), Orientation::Vertical);
    splitter.focus();
    splitter.press_key(Key::ArrowRight);
    assert!(near(sizes.get_untracked()[0], 0.26, 0.002));
    // 40 pixels of travel are 10% of the 400 the panes share.
    splitter.drag(Point::new(3.0, 50.0), Point::new(43.0, 50.0));
    assert!(
        near(sizes.get_untracked()[0], 0.36, 0.01),
        "{:?}",
        sizes.get_untracked()
    );
    clean(&screen);

    let (screen, sizes) = mount_split(Orientation::Vertical, vec![0.5, 0.5], 206.0);
    let splitter = screen.get_by_role(Role::Splitter, "Resize");
    assert_eq!(splitter.orientation(), Orientation::Horizontal);
    assert!(near(height(&splitter), 6.0, 0.6));
    splitter.focus();
    splitter.press_key(Key::ArrowDown);
    assert!(near(sizes.get_untracked()[0], 0.51, 0.002));
    splitter.press_key(Key::ArrowRight);
    assert!(
        near(sizes.get_untracked()[0], 0.51, 0.002),
        "only the axis' keys"
    );
    clean(&screen);
}

fn row_of(screen: &Screen, label: &str) -> Rect {
    screen
        .get_by_role(Role::Group, label)
        .rect()
        .expect("row is laid out")
}

#[test]
fn a_property_row_lays_out_label_leading_value_and_trailing_in_order() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            column((
                ui::property_row(
                    "Opacity",
                    kivact::ui::button("Value")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .test_id("value"),
                )
                .leading(|| {
                    kivact::ui::button("Lead")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .test_id("lead")
                })
                .trailing(|| {
                    kivact::ui::button("Trail")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .test_id("trail")
                }),
                ui::property_row(
                    "Plain",
                    kivact::ui::button("Only value").variant(kivact::ui::ButtonVariant::Secondary),
                ),
                ui::property_row(
                    "Wide",
                    kivact::ui::button("Wide value").variant(kivact::ui::ButtonVariant::Secondary),
                )
                .label_width(140.0)
                .row_height(40.0),
                ui::property_row("Short", node().style(style().height(10))).row_height(4.0),
            ))
            .style(style().width(400)),
        )
    });
    let label = screen.get_by_text("Opacity").rect().expect("label");
    let x = |id: &str| screen.get_by_test_id(id).rect().expect(id).x;
    assert!(
        near(label.width, 90.0, 0.6),
        "default label column: {label:?}"
    );
    assert!(label.x + label.width <= x("lead") + 0.5);
    assert!(x("lead") + 0.5 < x("value"));
    assert!(x("value") + 0.5 < x("trail"));
    let row = row_of(&screen, "Opacity");
    let value = screen.get_by_test_id("value").rect().expect("value");
    let trail = screen.get_by_test_id("trail").rect().expect("trail");
    assert!(
        near(trail.x + trail.width, row.x + row.width, 0.6),
        "trailing ends the row"
    );
    assert!(
        value.width > 150.0,
        "the value takes what is left: {value:?}"
    );
    assert!(near(row.height, 24.0, 0.6), "{row:?}");
    assert!(near(width(&screen.get_by_text("Wide")), 140.0, 0.6));
    assert!(near(row_of(&screen, "Wide").height, 40.0, 0.6));
    assert!(
        near(row_of(&screen, "Short").height, 20.0, 0.6),
        "never under 20"
    );
    // A row without slots is the label and the value.
    let only = screen
        .get_by_role(Role::Button, "Only value")
        .rect()
        .expect("only");
    assert!(near(only.x, 90.0 + 4.0, 0.6), "{only:?}");
    clean(&screen);
}

#[test]
fn the_label_is_one_line_and_a_tooltip_shows_all_of_it() {
    let label = "Subsurface Scattering Radius";
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), move || {
        (
            (),
            column((
                ui::property_row(
                    label,
                    kivact::ui::button("Value").variant(kivact::ui::ButtonVariant::Secondary),
                )
                .label_width(60.0),
                kivact::ui::button("Elsewhere").variant(kivact::ui::ButtonVariant::Secondary),
            ))
            .style(style().width(300)),
        )
    });
    let text = screen.get_by_text(label);
    let rect = text.rect().expect("label");
    assert!(
        near(rect.width, 60.0, 0.6),
        "clipped to its column: {rect:?}"
    );
    assert!(
        near(row_of(&screen, label).height, 24.0, 0.6),
        "stays on one line"
    );
    assert!(screen.query_by_role(Role::Tooltip, label).is_none());
    text.hover();
    screen.advance(Duration::from_millis(500));
    screen.get_by_role(Role::Tooltip, label);
    screen.mouse_leave();
    assert!(screen.query_by_role(Role::Tooltip, label).is_none());
    clean(&screen);
}

#[test]
fn state_buttons_name_the_state_and_draw_four_glyphs_in_the_theme_colors() {
    let (screen, (state, presses)) = kivact::test::render_with_handle(crate::options(), || {
        let state = signal(Plain);
        let presses = signal(0);
        (
            (state, presses),
            ui::property_state_button("Opacity", state)
                .on_press(move || presses.update(|n| *n += 1)),
        )
    });
    let button = || screen.get_by_role(Role::Button, "Opacity: Not animated");
    let rect = button().rect().expect("button");
    assert!(
        near(rect.width, 20.0, 0.6) && near(rect.height, 20.0, 0.6),
        "{rect:?}"
    );
    // Pixels of the button: the center, the outer corner of a 7 pixel square
    // (outside a diamond) and a pixel inside the square next to that corner.
    let glyph = |name: &str| {
        let shot = screen.get_by_role(Role::Button, name).screenshot();
        let at = |x, y| shot.pixel(x, y).expect("inside the button");
        (at(10, 10), at(6, 6), at(7, 7), shot.hash())
    };
    let accent = 0x8b7cff; // Iris
    let accent_high = 0xa397ff;
    let (background, outline, _, plain) = glyph("Opacity: Not animated");
    assert_ne!(outline, background, "an outlined square reaches the corner");
    state.set(Animated);
    screen.run_until_idle();
    let (center, outside, _, animated) = glyph("Opacity: Animated");
    assert_eq!(center, background, "an outline leaves the middle empty");
    assert_eq!(outside, background, "a diamond stays off the corner");
    state.set(Keyed);
    screen.run_until_idle();
    let (center, outside, inside, keyed) = glyph("Opacity: Keyframe");
    assert_eq!(center, accent_high, "filled, in the theme's highlight");
    assert_eq!(outside, background);
    assert_ne!(inside, center, "a diamond does not fill a square's corner");
    state.set(Driven);
    screen.run_until_idle();
    let (center, _, inside, driven) = glyph("Opacity: Driven");
    assert_eq!(center, accent, "filled, in the theme's accent");
    assert_eq!(inside, accent, "a square fills its corner");
    let all = [plain, animated, keyed, driven];
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            assert_ne!(a, b, "four different glyphs");
        }
    }

    screen.get_by_role(Role::Button, "Opacity: Driven").click();
    screen.get_by_role(Role::Button, "Opacity: Driven").focus();
    screen.press_key(Key::Enter);
    assert_eq!(presses.get_untracked(), 2);
    assert_eq!(
        state.get_untracked(),
        Driven,
        "the state is the application's"
    );
    clean(&screen);
}

#[test]
fn state_buttons_show_the_state_name_in_a_tooltip_and_read_their_messages() {
    let app = || {
        let state = signal(Plain);
        (
            state,
            column((
                ui::property_state_button("Opacity", state),
                kivact::ui::button("Elsewhere").variant(kivact::ui::ButtonVariant::Secondary),
            )),
        )
    };
    let (screen, state) = kivact::test::render_with_handle(crate::options(), app);
    let button = screen.get_by_role(Role::Button, "Opacity: Not animated");
    button.hover();
    screen.advance(Duration::from_millis(500));
    screen.get_by_role(Role::Tooltip, "Not animated");
    state.set(Keyed);
    screen.run_until_idle();
    screen.get_by_role(Role::Tooltip, "Keyframe");
    clean(&screen);

    let screen = render_with(
        RenderOptions {
            messages: Messages::new()
                .with("creative.property.state.plain", "固定")
                .with("creative.property.state.animated", "アニメーション")
                .with("creative.property.state.keyed", "キーフレーム")
                .with("creative.property.state.driven", "ドライバー"),
            ..RenderOptions::from_env()
        },
        move || {
            column((
                ui::property_state_button("不透明度", Keyed),
                ui::property_state_button("X", Driven),
            ))
        },
    );
    screen.get_by_role(Role::Button, "不透明度: キーフレーム");
    screen.get_by_role(Role::Button, "X: ドライバー");
    clean(&screen);
}

struct Window {
    open: Signal<bool>,
    anchor_shown: Signal<bool>,
    mounted: Signal<bool>,
    closes: Signal<usize>,
}

fn mount_window(options: ui::WindowOptions) -> (Screen, Window) {
    kivact::test::render_with_handle(crate::options(), move || {
        let open = signal(false);
        let anchor_shown = signal(true);
        let mounted = signal(true);
        let closes = signal(0);
        let anchor = node_ref();
        let options = options.clone();
        (
            Window {
                open,
                anchor_shown,
                mounted,
                closes,
            },
            column((
                show(anchor_shown, move || {
                    kivact::ui::button("Anchor")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .node_ref(anchor)
                        .expanded(open)
                        .on_press(move || open.set(true))
                }),
                kivact::ui::button("Other").variant(kivact::ui::ButtonVariant::Secondary),
                show(mounted, {
                    let options = options.clone();
                    move || {
                        ui::window(anchor, open, options.clone()).content(move |close| {
                            column((
                                kivact::ui::text_input("Field", signal(String::new()))
                                    .value_text("text"),
                                kivact::ui::button("Done")
                                    .variant(kivact::ui::ButtonVariant::Secondary)
                                    .on_press(move || {
                                        closes.update(|n| *n += 1);
                                        close.close();
                                    }),
                            ))
                        })
                    }
                }),
            )),
        )
    })
}

#[test]
fn an_application_opens_and_closes_a_window_through_its_binding() {
    let (screen, window) = mount_window(ui::WindowOptions::default().title("Tool"));
    assert!(screen.query_by_role(Role::Dialog, "Tool").is_none());
    window.open.set(true);
    screen.run_until_idle();
    screen.get_by_role(Role::Dialog, "Tool");
    screen.get_by_role(Role::Group, "Tool window");
    window.open.set(false);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Dialog, "Tool").is_none());

    // Every way to close writes false.
    for close in 0..4 {
        window.open.set(true);
        screen.run_until_idle();
        screen.get_by_role(Role::TextBox, "Field").focus();
        match close {
            0 => screen.get_by_role(Role::Button, "Close window").click(),
            1 => screen.press_key(Key::Escape),
            2 => screen
                .get_by_role(Role::Dialog, "Tool")
                .a11y(A11yAction::Dismiss),
            _ => screen.get_by_role(Role::Button, "Done").click(),
        }
        assert!(!window.open.get_untracked(), "way {close}");
        assert!(screen.query_by_role(Role::Dialog, "Tool").is_none());
        assert!(
            screen.get_by_role(Role::Button, "Anchor").is_focused(),
            "way {close}: nonmodal windows give the focus to the anchor"
        );
    }
    assert_eq!(window.closes.get_untracked(), 1);

    // An outside press closes it but leaves the focus where it was pressed.
    window.open.set(true);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Other").click();
    assert!(!window.open.get_untracked());
    assert!(screen.get_by_role(Role::Button, "Other").is_focused());
    clean(&screen);
}

#[test]
fn removing_a_modeless_window_anchor_closes_it_without_disturbing_other_focus() {
    let (screen, window) = mount_window(ui::WindowOptions::default().title("Tool"));
    screen.get_by_role(Role::Button, "Other").focus();
    window.open.set(true);
    screen.run_until_idle();
    assert!(screen.get_by_role(Role::Dialog, "Tool").exists());
    window.anchor_shown.set(false);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Button, "Anchor").is_none());
    assert!(!window.open.get_untracked());
    assert!(screen.query_by_role(Role::Dialog, "Tool").is_none());
    assert!(screen.get_by_role(Role::Button, "Other").is_focused());
    clean(&screen);
}

#[test]
fn a_modal_window_gives_the_focus_back_to_the_element_that_had_it() {
    let (screen, window) = mount_window(ui::WindowOptions::default().title("Confirm").modal(true));
    // The anchor does not have the focus; the other button does.
    screen.get_by_role(Role::Button, "Other").focus();
    window.open.set(true);
    screen.run_until_idle();
    screen.get_by_role(Role::Dialog, "Confirm");
    assert!(
        screen
            .query_by_role(Role::Button, "Minimize window")
            .is_none(),
        "a modal window does not collapse"
    );
    screen.get_by_role(Role::Button, "Done").click();
    assert!(!window.open.get_untracked());
    assert!(screen.get_by_role(Role::Button, "Other").is_focused());

    window.open.set(true);
    screen.run_until_idle();
    screen.press_key(Key::Escape);
    assert!(!window.open.get_untracked());
    assert!(screen.get_by_role(Role::Button, "Other").is_focused());

    // A press behind the modal is not an outside press.
    window.open.set(true);
    screen.run_until_idle();
    let behind = screen
        .get_by_role(Role::Button, "Other")
        .rect()
        .expect("other");
    screen.mouse_move(behind.center().x, behind.center().y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert!(window.open.get_untracked());
    screen.get_by_role(Role::Dialog, "Confirm");
    clean(&screen);
}

#[test]
fn a_window_starts_expanded_and_in_place_at_every_opening() {
    let (screen, window) = mount_window(ui::WindowOptions::default().title("Tool"));
    window.open.set(true);
    screen.run_until_idle();
    let window_box = || screen.get_by_role(Role::Group, "Tool window");
    let title = || screen.get_by_role(Role::Button, "Tool title bar");
    let original = window_box().rect().expect("window");
    title().drag(Point::new(20.0, 15.0), Point::new(80.0, 75.0));
    let moved = window_box().rect().expect("moved");
    assert!(
        near(moved.x - original.x, 60.0, 0.6) && near(moved.y - original.y, 60.0, 0.6),
        "{original:?} {moved:?}"
    );
    title().a11y(A11yAction::Toggle);
    assert_eq!(title().is_expanded(), Some(false));

    window.open.set(false);
    screen.run_until_idle();
    window.open.set(true);
    screen.run_until_idle();
    assert_eq!(title().is_expanded(), Some(true), "expanded again");
    assert_eq!(window_box().rect(), Some(original), "back in place");
    clean(&screen);
}

#[test]
fn a_window_whose_part_is_unmounted_writes_false_to_open() {
    let (screen, window) = mount_window(ui::WindowOptions::default().title("Tool"));
    window.open.set(true);
    screen.run_until_idle();
    window.mounted.set(false);
    screen.run_until_idle();
    assert!(!window.open.get_untracked());
    assert!(screen.query_by_role(Role::Dialog, "Tool").is_none());
    clean(&screen);
}

#[test]
fn a_window_without_a_title_option_is_named_by_a_message() {
    let (screen, window) = mount_window(ui::WindowOptions::default());
    window.open.set(true);
    screen.run_until_idle();
    screen.get_by_role(Role::Dialog, "Window");
    clean(&screen);
}

#[test]
fn a_panel_header_replaces_the_default_one_and_keeps_its_height() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            row((
                ui::panel("Plain", node().test_id("plain-body"))
                    .style(style().width(200).height(150)),
                ui::panel("Custom", node().test_id("body").style(style().height(500)))
                    .header(|| {
                        row((
                            text("Toolbar"),
                            kivact::ui::button("Add").variant(kivact::ui::ButtonVariant::Secondary),
                        ))
                        .test_id("header")
                        .style(style().height(40))
                    })
                    .child(text("Footer"))
                    .style(style().width(200).height(150)),
            ))
            .style(style().gap(10)),
        )
    });
    let plain = screen.get_by_role(Role::Group, "Plain");
    screen.get_by_text("Plain");
    let body = screen.get_by_test_id("plain-body").rect().expect("body");
    assert!(
        // The 1 px border, then the 36 px title bar.
        near(body.y - plain.rect().expect("panel").y, 37.0, 0.6),
        "{body:?}"
    );

    let panel = screen.get_by_role(Role::Group, "Custom");
    assert!(
        screen.query_by_text("Custom").is_none(),
        "no second title bar"
    );
    let header = screen.get_by_test_id("header").rect().expect("header");
    assert!(
        near(header.height, 40.0, 0.6),
        "the header does not shrink: {header:?}"
    );
    screen.get_by_role(Role::Button, "Add");
    let body = screen.get_by_test_id("body").rect().expect("body");
    assert!(
        near(body.y, header.y + 40.0, 0.6),
        "the content follows the header"
    );
    assert!(panel.rect().expect("panel").height <= 150.5);
    clean(&screen);
}
