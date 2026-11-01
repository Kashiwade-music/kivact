//! Role: Color wheel model inputs, accessible targets and edit cancellation.
use super::*;
use kivact::style::Color;
use kivact::ui::EditPhase::{Begin, Commit};

#[test]
fn color_wheel_regions_keys_actions_and_gray_hue_preservation() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, color) = kivact::test::render_with_handle(crate::options(), move || {
        let color = signal(Color::from_rgba_f32(0.5, 0.5, 0.5, 0.4));
        (
            color,
            ui::color_wheel(color, signal(ui::ColorModel::Rgb))
                .on_edit(move |phase| seen.borrow_mut().push(phase)),
        )
    });
    let hue = screen.get_by_role(Role::Slider, "Hue");
    for action in [
        A11yAction::Select,
        A11yAction::Activate,
        A11yAction::SetValue,
        A11yAction::Increment,
        A11yAction::Decrement,
    ] {
        assert!(hue.a11y_actions().contains(&action));
    }
    hue.a11y_with_value(A11yAction::SetValue, "120");
    assert_eq!(*log.borrow(), [Begin, Commit]);
    log.borrow_mut().clear();
    assert_eq!(color.get_untracked().components()[0], 0.5);
    let sv = screen.get_by_role(Role::Slider, "Saturation and value");
    sv.a11y_with_value(A11yAction::SetValue, "1,0.5");
    assert_eq!(*log.borrow(), [Begin, Commit]);
    log.borrow_mut().clear();
    let (h, s, v) = ui::color_to_hsv(color.get_untracked());
    assert!((h - 120.0).abs() < 0.01);
    assert!((s - 1.0).abs() < 0.01);
    assert!((v - 0.5).abs() < 0.01);
    hue.press_key(Key::ArrowRight);
    assert_eq!(*log.borrow(), [Begin, Commit]);
    log.borrow_mut().clear();
    screen.press_key_with(
        Key::ArrowUp,
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );
    assert_eq!(*log.borrow(), [Begin, Commit]);
    log.borrow_mut().clear();
    assert!((ui::color_to_hsv(color.get_untracked()).0 - 131.0).abs() < 0.01);
    let alpha = screen.get_by_role(Role::Slider, "Alpha");
    alpha.a11y_with_value(A11yAction::SetValue, "0.7");
    assert!((color.get_untracked().components()[3] - 0.7).abs() < 0.001);
    assert_eq!(*log.borrow(), [Begin, Commit]);
    log.borrow_mut().clear();
    sv.a11y_with_value(A11yAction::SetValue, "NaN,0.5");
    hue.a11y_with_value(A11yAction::SetValue, "inf");
    assert!(log.borrow().is_empty());
    clean(&screen);
}
