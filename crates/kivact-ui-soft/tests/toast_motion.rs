//! Role: Verify Soft's animated rows preserve the standard toast lifecycle.
use kivact::control::column;
use kivact::style::ThemeMode;
use kivact::test::{RenderOptions, WindowMode, render_with_handle};
use kivact::ui::{Toast, ToastOptions, button, toast_region, toaster};
use kivact::view::{Key, Role};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

fn options() -> RenderOptions {
    RenderOptions {
        theme: kivact_ui_soft::theme(ThemeMode::Light),
        window: WindowMode::Headless,
        ..RenderOptions::default()
    }
}

#[test]
fn dismissing_a_toast_removes_its_controls_before_the_row_collapses() {
    let hidden = Rc::new(Cell::new(0));
    let count = hidden.clone();
    let (screen, _) = render_with_handle(options(), move || {
        let toasts = toaster();
        toasts.push(Toast::new("Earlier").duration(None));
        toasts.push_with(
            Toast::new("Dismiss me").duration(None),
            ToastOptions::default().on_hide(move || count.set(count.get() + 1)),
        );
        ((), column((button("Outside"), toast_region(toasts))))
    });
    screen.advance(Duration::from_millis(180));
    let earlier_y = || {
        screen
            .get_by_role(Role::Status, "Earlier")
            .rect()
            .unwrap()
            .y
    };
    let expanded = earlier_y();
    screen.get_by_role(Role::Button, "Outside").focus();
    screen
        .get_by_role(Role::Status, "Dismiss me")
        .get_by_role(Role::Button, "Close")
        .focus();
    screen.press_key(Key::Escape);
    assert_eq!(hidden.get(), 1);
    assert!(screen.query_by_role(Role::Status, "Dismiss me").is_none());
    assert_eq!(screen.focused().unwrap().name(), "Outside");
    screen.advance(Duration::from_millis(48));
    let halfway = earlier_y();
    screen.advance(Duration::from_millis(130));
    assert!(expanded < halfway && halfway < earlier_y());
    assert_eq!(hidden.get(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn toast_row_does_not_extend_the_auto_dismiss_timer() {
    let hidden = Rc::new(Cell::new(0));
    let count = hidden.clone();
    let (screen, _) = render_with_handle(options(), move || {
        let toasts = toaster();
        toasts.push_with(
            Toast::new("Automatic").duration(Some(Duration::from_millis(240))),
            ToastOptions::default().on_hide(move || count.set(count.get() + 1)),
        );
        ((), toast_region(toasts))
    });
    screen.advance(Duration::from_millis(210));
    assert!(screen.query_by_role(Role::Status, "Automatic").is_some());
    screen.advance(Duration::from_millis(65));
    assert!(screen.query_by_role(Role::Status, "Automatic").is_none());
    assert_eq!(hidden.get(), 1);
    screen.advance(Duration::from_millis(180));
    assert_eq!(hidden.get(), 1);
}
