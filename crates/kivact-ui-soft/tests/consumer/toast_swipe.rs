//! Role: Exercise Soft toast swipes, pointer filtering, and the timer hold.

use kivact::interaction::Toast;
use kivact::interaction::toaster;
use kivact::style::ThemeMode;
use kivact::test::RenderOptions;
use kivact::test::render_with;
use kivact::ui::toast_region;
use kivact::view::MouseButton;
use kivact::view::PointerKind;
use kivact::view::Role;
use std::time::Duration;

#[test]
fn a_short_or_mouse_drag_does_not_dismiss_but_an_edge_swipe_does() {
    for kind in [PointerKind::Touch, PointerKind::Pen] {
        let screen = render_with(
            RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)),
            || {
                let queue = toaster();
                queue.push(Toast::new("Saved").duration(None));
                toast_region(queue)
            },
        );
        screen.advance(Duration::from_millis(150));
        let card = || screen.get_by_role(Role::Status, "Saved");
        let rect = card().rect().unwrap();
        let x = rect.x + 25.0;
        let y = rect.y + 5.0;

        screen.mouse_move(x, y);
        screen.mouse_down(MouseButton::Primary);
        screen.mouse_move(x, y + 52.0);
        screen.mouse_up(MouseButton::Primary);
        card();

        let close = screen.get_by_role(Role::Button, "Close").rect().unwrap();
        let close_x = close.x + close.width / 2.0;
        let close_y = close.y + close.height / 2.0;
        let contact = screen.contact_down(kind, close_x, close_y);
        screen.contact_move(contact, close_x, close_y + 52.0);
        screen.contact_up(contact);
        card();

        let contact = screen.contact_down(kind, x, y);
        screen.contact_move(contact, x, y - 52.0);
        screen.contact_up(contact);
        card();

        let contact = screen.contact_down(kind, x, y);
        screen.contact_move(contact, x + 15.0, y + 2.0);
        screen.contact_move(contact, x, y + 52.0);
        screen.contact_up(contact);
        card();

        let contact = screen.contact_down(kind, x, y);
        screen.contact_move(contact, x, y + 25.0);
        screen.contact_up(contact);
        screen.advance(Duration::from_millis(150));
        card();

        let contact = screen.contact_down(kind, x, y);
        screen.contact_move(contact, x, y + 52.0);
        screen.contact_up(contact);
        screen.advance(Duration::from_millis(32));
        assert!(screen.query_by_role(Role::Status, "Saved").is_none());
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[test]
fn a_pending_touch_holds_only_its_toasts_timer() {
    let screen = render_with(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)),
        || {
            let queue = toaster();
            for title in ["Held", "Other"] {
                queue.push(Toast::new(title).duration(Some(Duration::from_millis(350))));
            }
            toast_region(queue)
        },
    );
    screen.advance(Duration::from_millis(150));
    let card = screen.get_by_role(Role::Status, "Held").rect().unwrap();
    let contact = screen.contact_down(PointerKind::Touch, card.x + 24.0, card.y + 8.0);
    screen.advance(Duration::from_millis(400));
    assert!(screen.query_by_role(Role::Status, "Held").is_some());
    assert!(screen.query_by_role(Role::Status, "Other").is_none());

    screen.contact_cancel(contact);
    screen.advance(Duration::from_millis(1200));
    assert!(screen.query_by_role(Role::Status, "Held").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
