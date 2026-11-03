//! Role: Verify Soft switch pointer gestures.
use kivact::binding;
use kivact::signal;
use kivact::style::ThemeMode;
use kivact::test::{RenderOptions, render_with_handle};
use kivact::ui::switch;
use kivact::view::{Key, PointerKind, Role};

#[test]
fn switch_touch_commits_once_and_vertical_or_cancelled_contacts_do_not_toggle() {
    let (screen, (on, writes)) = render_with_handle(
        RenderOptions {
            theme: kivact_ui_soft::theme(ThemeMode::Light),
            ..RenderOptions::from_env()
        },
        || {
            let on = signal(false);
            let writes = signal(0);
            let value = binding(on, move |next| {
                writes.update(|count| *count += 1);
                on.set(next);
            });
            ((on, writes), switch("Notifications", value))
        },
    );
    let control = screen.get_by_role(Role::Switch, "Notifications");
    let bounds = control.accessibility_bounds().unwrap();
    let x = bounds.x + 14.0;
    let y = bounds.y + bounds.height / 2.0;
    let touch = || screen.contact_down(PointerKind::Touch, x, y);

    let finger = touch();
    assert!(!on.get_untracked());
    screen.contact_up(finger);
    assert!(on.get_untracked());
    assert_eq!(writes.get_untracked(), 1);

    let finger = touch();
    screen.contact_move(finger, x, y + 24.0);
    screen.contact_up(finger);
    assert!(on.get_untracked());
    assert_eq!(writes.get_untracked(), 1);

    let finger = touch();
    screen.contact_move(finger, x + 30.0, y);
    screen.contact_cancel(finger);
    assert!(on.get_untracked());
    assert_eq!(writes.get_untracked(), 1);

    control.press_key(Key::Space);
    assert!(!on.get_untracked());
    assert_eq!(writes.get_untracked(), 2);
    control.press_key(Key::Enter);
    assert!(!on.get_untracked());
    assert_eq!(writes.get_untracked(), 2);

    let finger = touch();
    screen.contact_move(finger, x + 28.0, y);
    screen.contact_move(finger, x + 30.0, y);
    assert!(!on.get_untracked());
    screen.contact_move(finger, x - 3.0, y);
    screen.contact_up(finger);
    assert!(
        !on.get_untracked(),
        "a drag that flips and returns keeps the starting side"
    );
    assert_eq!(writes.get_untracked(), 2);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
