//! Role: Reactive color wheel options and cancellation when controls disappear.
use super::*;
use kivact::style::Color;
use kivact::ui::EditPhase::{Begin, Cancel};

#[test]
fn color_wheel_options_react_and_disappearing_alpha_cancels_drag() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let (screen, (color, fields, alpha, size)) =
        kivact::test::render_with_handle(crate::options(), move || {
            let color = signal(Color::from_rgba_f32(0.4, 0.6, 0.2, 0.8));
            let fields = signal(true);
            let alpha = signal(true);
            let size = signal(220.0_f32);
            (
                (color, fields, alpha, size),
                ui::color_wheel(color, signal(ui::ColorModel::Rgb))
                    .fields(fields)
                    .alpha(alpha)
                    .wheel_size(size)
                    .on_edit(move |phase| seen.borrow_mut().push(phase)),
            )
        });
    screen.get_by_role(Role::SpinButton, "Color R");
    let wheel = screen.get_by_role(Role::Canvas, "Color wheel");
    assert_eq!(wheel.rect().unwrap().height, 220.0);
    fields.set(false);
    size.set(20.0);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::SpinButton, "Color R").is_none());
    assert_eq!(wheel.rect().unwrap().height, 96.0);
    size.set(f32::NAN);
    screen.run_until_idle();
    assert_eq!(wheel.rect().unwrap().height, 220.0);
    let original = color.get_untracked();
    let area = screen.get_by_role(Role::Slider, "Alpha").rect().unwrap();
    let start = area.center();
    screen.mouse_move(start.x, start.y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(start.x + 20.0, start.y);
    assert_ne!(color.get_untracked(), original);
    alpha.set(false);
    screen.run_until_idle();
    assert_eq!(color.get_untracked(), original);
    assert_eq!(*log.borrow(), vec![Begin, Cancel]);
    assert!(screen.query_by_role(Role::Slider, "Alpha").is_none());
    screen.mouse_up(MouseButton::Primary);
    fields.set(true);
    alpha.set(true);
    size.set(300.0);
    screen.run_until_idle();
    screen.get_by_role(Role::SpinButton, "Color Alpha");
    screen.get_by_role(Role::Slider, "Alpha");
    assert_eq!(wheel.rect().unwrap().height, 300.0);
    assert!(screen.errors().is_empty());
    clean(&screen);
}

#[test]
fn hiding_an_edited_field_restores_its_original_color() {
    for hide_alpha in [false, true] {
        let log = Rc::new(RefCell::new(Vec::new()));
        let seen = log.clone();
        let (screen, (color, fields, alpha)) =
            kivact::test::render_with_handle(crate::options(), move || {
                let color = signal(Color::from_rgba_f32(0.4, 0.6, 0.2, 0.8));
                let fields = signal(true);
                let alpha = signal(true);
                (
                    (color, fields, alpha),
                    ui::color_wheel(color, signal(ui::ColorModel::Rgb))
                        .fields(fields)
                        .alpha(alpha)
                        .on_edit(move |phase| seen.borrow_mut().push(phase)),
                )
            });
        let original = color.get_untracked();
        let name = if hide_alpha { "Color Alpha" } else { "Color R" };
        let area = screen.get_by_role(Role::SpinButton, name).rect().unwrap();
        let start = area.center();
        screen.mouse_move(start.x, start.y);
        screen.mouse_down(MouseButton::Primary);
        screen.mouse_move(start.x + 30.0, start.y);
        assert_ne!(color.get_untracked(), original);
        if hide_alpha {
            alpha.set(false);
        } else {
            fields.set(false);
        }
        screen.run_until_idle();
        assert_eq!(color.get_untracked(), original);
        assert_eq!(*log.borrow(), vec![Begin, Cancel]);
        screen.mouse_up(MouseButton::Primary);
        clean(&screen);
    }
}
