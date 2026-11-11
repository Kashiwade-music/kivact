//! Role: Exercise the Soft slider's touch-specific edit and thumb motion.

use kivact::state::signal;
use kivact::style::ThemeMode;
use kivact::style::style;
use kivact::test::RenderOptions;
use kivact::test::render_with_handle;
use kivact::ui::slider;
use kivact::view::EditPhase;
use kivact::view::Element;
use kivact::view::MouseButton;
use kivact::view::PointerKind;
use kivact::view::Role;
use std::time::Duration;

#[test]
fn touch_slop_and_relative_tracking_keep_one_edit_through_release_coast() {
    let (screen, (value, phases)) = render_with_handle(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)),
        || {
            let value = signal(50.0);
            let phases = signal(Vec::new());
            let slider = slider("Amount", value)
                .on_edit(move |phase| phases.update(|events| events.push(phase)))
                .style(style().width(200));
            ((value, phases), slider)
        },
    );
    let thumb = screen.get_by_role(Role::Slider, "Amount").rect().unwrap();
    let x = thumb.x + thumb.width / 2.0;
    let y = thumb.y + thumb.height / 2.0;
    let contact = screen.contact_down(PointerKind::Touch, x, y);
    screen.advance(Duration::from_millis(20));
    screen.contact_move(contact, x + 5.0, y);
    assert_eq!(value.get(), 50.0);
    assert!(phases.get().is_empty());

    screen.advance(Duration::from_millis(30));
    screen.contact_move(contact, x + 40.0, y);
    assert!((value.get() - 65.0).abs() < 0.01);
    assert_eq!(phases.get(), vec![EditPhase::Begin]);
    screen.contact_up(contact);
    assert_eq!(phases.get(), vec![EditPhase::Begin]);
    screen.advance(Duration::from_millis(1600));
    assert!(value.get() > 65.0 && value.get() <= 100.0);
    assert_eq!(phases.get(), vec![EditPhase::Begin, EditPhase::Commit]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn track_touch_only_changes_on_tap_release_and_mouse_still_jumps_on_press() {
    let (screen, value) = render_with_handle(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)),
        || {
            let value = signal(50.0);
            (value, slider("Amount", value).style(style().width(200)))
        },
    );
    let thumb = screen.get_by_role(Role::Slider, "Amount").rect().unwrap();
    let y = thumb.y + thumb.height / 2.0;
    let x = thumb.x + thumb.width / 2.0 + 50.0;
    let contact = screen.contact_down(PointerKind::Touch, x, y);
    assert_eq!(value.get(), 50.0);
    screen.contact_move(contact, x, y + 20.0);
    screen.contact_up(contact);
    assert_eq!(value.get(), 50.0);
    let contact = screen.contact_down(PointerKind::Touch, x, y);
    assert_eq!(value.get(), 50.0);
    screen.contact_up(contact);
    assert!(value.get() > 70.0);
    screen.mouse_move(x - 100.0, y);
    screen.mouse_down(MouseButton::Primary);
    assert!(value.get() < 30.0);
    screen.mouse_up(MouseButton::Primary);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn touch_thumb_lifts_and_drops_except_under_reduced_motion() {
    for reduced in [false, true] {
        let (screen, value) = render_with_handle(
            if reduced {
                RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)).reduced_motion()
            } else {
                RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light))
            },
            || {
                let value = signal(50.0);
                (value, slider("Amount", value).style(style().width(200)))
            },
        );
        let slider = screen.get_by_role(Role::Slider, "Amount");
        let thumb = slider.accessibility_bounds().unwrap();
        let x = thumb.x + thumb.width / 2.0;
        let y = thumb.y + thumb.height / 2.0;
        let width = || slider.accessibility_bounds().unwrap().width;
        let contact = screen.contact_down(PointerKind::Touch, x, y);
        screen.advance(Duration::from_millis(140));
        if reduced {
            assert!((width() - thumb.width).abs() < 0.5);
        } else {
            assert!(width() > thumb.width * 1.4);
        }
        screen.contact_up(contact);
        screen.advance(Duration::from_millis(950));
        assert!((width() - thumb.width).abs() < 0.5);
        assert_eq!(value.get(), 50.0);
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
