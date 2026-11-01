//! Role: `Numeric`'s edit phases, text parser, mixed value and reactive property state.

use super::*;
use kivact::i18n::Messages;
use kivact::ui::EditPhase::{self, Begin, Cancel, Commit};
use kivact::view::PointerKind;

/// The phases an observer saw, each with the value in the binding at that time.
type Log = Rc<RefCell<Vec<(EditPhase, f64)>>>;

fn observer(value: Signal<f64>, log: &Log) -> impl Fn(EditPhase) + 'static {
    let log = log.clone();
    move |phase| log.borrow_mut().push((phase, value.get_untracked()))
}

fn center(screen: &Screen, role: Role, name: &str) -> (f32, f32) {
    let rect = screen.get_by_role(role, name).rect().expect("laid out");
    (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
}

fn gain(log: &Log) -> (Screen, Signal<f64>) {
    let seen = log.clone();
    kivact::test::render_with_handle(crate::options(), move || {
        let value = signal(10.0);
        (
            value,
            ui::number("Gain", value)
                .range(0.0, 100.0)
                .step(1.0)
                .precision(1)
                .on_edit(observer(value, &seen)),
        )
    })
}

#[test]
fn a_scrub_is_one_edit_from_press_to_release() {
    let log = Log::default();
    let (screen, value) = gain(&log);
    let (x, y) = center(&screen, Role::SpinButton, "Gain");
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(*log.borrow(), [(Begin, 10.0)], "before the first write");
    screen.mouse_move(x + 10.0, y);
    screen.mouse_move(x + 20.0, y);
    assert_eq!(value.get_untracked(), 30.0);
    assert_eq!(log.borrow().len(), 1, "the moves are not edits");
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(*log.borrow(), [(Begin, 10.0), (Commit, 30.0)]);
    assert!(
        screen.query_by_role(Role::TextBox, "Gain").is_none(),
        "a scrub does not open the text field"
    );
    clean(&screen);
}

#[test]
fn escape_during_a_scrub_restores_the_start_and_cancels() {
    let log = Log::default();
    let (screen, value) = gain(&log);
    let (x, y) = center(&screen, Role::SpinButton, "Gain");
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 20.0, y);
    assert_eq!(value.get_untracked(), 30.0);
    screen.press_key(Key::Escape);
    assert_eq!(*log.borrow(), [(Begin, 10.0), (Cancel, 10.0)]);
    assert_eq!(value.get_untracked(), 10.0);
    screen.mouse_move(x + 40.0, y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(log.borrow().len(), 2, "the rest of the gesture is silent");
    assert_eq!(value.get_untracked(), 10.0);

    log.borrow_mut().clear();
    let contact = screen.contact_down(PointerKind::Touch, x, y);
    screen.contact_move(contact, x + 20.0, y);
    assert_eq!(value.get_untracked(), 30.0);
    screen.contact_cancel(contact);
    assert_eq!(
        *log.borrow(),
        [(Begin, 10.0), (Cancel, 10.0)],
        "the engine canceling the pointer is a Cancel"
    );
    assert_eq!(value.get_untracked(), 10.0);
    clean(&screen);
}

#[test]
fn each_key_wheel_and_assistive_technology_write_is_its_own_edit() {
    let log = Log::default();
    let (screen, value) = gain(&log);
    let gain = screen.get_by_role(Role::SpinButton, "Gain");
    gain.focus();
    for (key, expected) in [
        (Key::ArrowRight, 11.0),
        (Key::ArrowDown, 10.0),
        (Key::End, 100.0),
        (Key::Home, 0.0),
        (Key::Backspace, 0.0),
    ] {
        let before = value.get_untracked();
        log.borrow_mut().clear();
        screen.press_key(key);
        assert_eq!(value.get_untracked(), expected, "{key:?}");
        assert_eq!(
            *log.borrow(),
            [(Begin, before), (Commit, expected)],
            "{key:?}"
        );
    }
    log.borrow_mut().clear();
    gain.scroll(0.0, -1.0);
    assert_eq!(value.get_untracked(), 1.0);
    assert_eq!(
        *log.borrow(),
        [(Begin, 0.0), (Commit, 1.0)],
        "one wheel notch"
    );
    log.borrow_mut().clear();
    gain.a11y(A11yAction::Increment);
    gain.a11y_with_value(A11yAction::SetValue, "40");
    assert_eq!(
        *log.borrow(),
        [(Begin, 1.0), (Commit, 2.0), (Begin, 2.0), (Commit, 40.0)]
    );
    log.borrow_mut().clear();
    gain.a11y_with_value(A11yAction::SetValue, "NaN");
    gain.a11y_with_value(A11yAction::SetValue, "no value");
    assert!(log.borrow().is_empty(), "{:?}", log.borrow());
    clean(&screen);
}

#[test]
fn confirming_text_is_one_edit_and_discarding_it_sends_nothing() {
    let log = Log::default();
    let (screen, value) = gain(&log);
    let gain = screen.get_by_role(Role::SpinButton, "Gain");
    gain.press_key(Key::Enter);
    assert!(log.borrow().is_empty(), "opening the field is not an edit");
    screen.get_by_role(Role::TextBox, "Gain").type_text("2 * 8");
    assert!(log.borrow().is_empty(), "typing is not an edit");
    screen.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), 16.0);
    assert_eq!(*log.borrow(), [(Begin, 10.0), (Commit, 16.0)]);

    log.borrow_mut().clear();
    gain.press_key(Key::Enter);
    screen.get_by_role(Role::TextBox, "Gain").type_text("99");
    screen.press_key(Key::Escape);
    assert_eq!(value.get_untracked(), 16.0);
    assert!(log.borrow().is_empty(), "Escape discards the text silently");

    gain.press_key(Key::Enter);
    screen.get_by_role(Role::TextBox, "Gain").type_text("5");
    screen.get_by_role(Role::SpinButton, "Gain").focus();
    assert_eq!(value.get_untracked(), 5.0, "leaving the field confirms it");
    assert_eq!(*log.borrow(), [(Begin, 16.0), (Commit, 5.0)]);
    clean(&screen);
}

#[test]
fn a_text_parser_runs_while_typing_and_a_refused_text_keeps_the_draft() {
    let log = Log::default();
    let starts = Rc::new(RefCell::new(Vec::new()));
    let seen = log.clone();
    let seen_starts = starts.clone();
    let (screen, value) = kivact::test::render_with_handle(crate::options(), move || {
        let value = signal(10.0);
        (
            value,
            column((
                ui::number("Gain", value)
                    .range(0.0, 100.0)
                    .on_edit(observer(value, &seen))
                    .text_parser(move |text, start| {
                        seen_starts.borrow_mut().push(start);
                        match text.trim() {
                            "inf" => Ok(f64::INFINITY),
                            text => text
                                .parse::<f64>()
                                .map(|v| start + v)
                                .map_err(|_| format!("\"{text}\" is not an offset")),
                        }
                    }),
                kivact::ui::button("Outside").variant(kivact::ui::ButtonVariant::Secondary),
            )),
        )
    });
    let gain = screen.get_by_role(Role::SpinButton, "Gain");
    gain.press_key(Key::Enter);
    let field = screen.get_by_role(Role::TextBox, "Gain");
    assert!(!field.invalid(), "the starting text is the number");
    field.type_text("abc");
    assert!(field.invalid(), "the parser runs while typing");
    let reason = screen.get_by_role(Role::Status, "\"abc\" is not an offset");
    assert_eq!(
        field.described_by().len(),
        1,
        "the reason is the field's description"
    );
    assert!(reason.is_visible());
    assert_eq!(starts.borrow().last(), Some(&10.0), "the start value");

    screen.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), 10.0, "nothing is written");
    assert!(log.borrow().is_empty());
    assert!(field.exists() && field.invalid(), "the field stays open");

    screen.get_by_role(Role::Button, "Outside").click();
    assert!(
        screen.query_by_role(Role::TextBox, "Gain").is_some(),
        "blurring keeps the draft"
    );
    assert_eq!(
        screen.get_by_role(Role::TextBox, "Gain").value().as_deref(),
        Some("abc")
    );
    assert_eq!(value.get_untracked(), 10.0);
    assert!(log.borrow().is_empty());

    let field = screen.get_by_role(Role::TextBox, "Gain");
    field.press_key_with(Key::Char('a'), Modifiers::default());
    field.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    field.type_text("inf");
    assert!(field.invalid());
    screen.get_by_role(Role::Status, "Enter a finite number");
    screen.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), 10.0, "a non-finite value is refused");

    field.press_key_with(
        Key::Char('a'),
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    field.type_text("5");
    assert!(!field.invalid());
    assert!(
        screen
            .query_by_role(Role::Status, "Enter a finite number")
            .is_none(),
        "the reason goes with the error"
    );
    screen.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), 15.0, "relative to the start value");
    assert_eq!(*log.borrow(), [(Begin, 10.0), (Commit, 15.0)]);
    assert!(screen.query_by_role(Role::TextBox, "Gain").is_none());

    gain.press_key(Key::Enter);
    screen.get_by_role(Role::TextBox, "Gain").type_text("zzz");
    screen.get_by_role(Role::Button, "Outside").click();
    screen
        .get_by_role(Role::TextBox, "Gain")
        .press_key(Key::Escape);
    assert!(screen.query_by_role(Role::TextBox, "Gain").is_none());
    assert_eq!(value.get_untracked(), 15.0, "Escape discards the draft");
    assert_eq!(log.borrow().len(), 2);
    clean(&screen);
}

#[test]
fn the_built_in_parser_explains_a_text_it_cannot_read() {
    let (screen, value) = kivact::test::render_with_handle(crate::options(), || {
        let value = signal(1.0);
        (value, ui::number("Gain", value).range(0.0, 10.0))
    });
    screen
        .get_by_role(Role::SpinButton, "Gain")
        .press_key(Key::Enter);
    let field = screen.get_by_role(Role::TextBox, "Gain");
    field.type_text("1 / 0");
    assert!(field.invalid());
    screen.get_by_role(Role::Status, "Enter a finite number");
    screen.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), 1.0);
    assert!(field.exists());
    clean(&screen);
}

fn mixed_gain(log: &Log) -> (Screen, (Signal<f64>, Signal<bool>)) {
    let seen = log.clone();
    kivact::test::render_with_handle(crate::options(), move || {
        let value = signal(10.0);
        let mixed = signal(true);
        (
            (value, mixed),
            ui::number("Gain", value)
                .range(0.0, 100.0)
                .step(1.0)
                .mixed(mixed)
                .on_edit(observer(value, &seen)),
        )
    })
}

#[test]
fn a_mixed_value_shows_mixed_and_takes_only_typed_text_and_set_value() {
    let log = Log::default();
    let (screen, (value, mixed)) = mixed_gain(&log);
    let gain = screen.get_by_role(Role::SpinButton, "Gain");
    assert!(gain.is_indeterminate());
    assert_eq!(gain.value().as_deref(), Some("Mixed"));
    assert!(screen.get_by_text("Mixed").exists());

    let (x, y) = center(&screen, Role::SpinButton, "Gain");
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 30.0, y);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(value.get_untracked(), 10.0, "no scrub");
    assert!(log.borrow().is_empty());
    let field = screen.get_by_role(Role::TextBox, "Gain");
    assert!(field.is_focused(), "a primary press starts the text edit");
    screen.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::TextBox, "Gain").is_none());

    let gain = screen.get_by_role(Role::SpinButton, "Gain");
    gain.focus();
    for key in [
        Key::ArrowRight,
        Key::ArrowUp,
        Key::Home,
        Key::End,
        Key::Backspace,
    ] {
        screen.press_key(key);
    }
    gain.scroll(0.0, -1.0);
    gain.double_click();
    assert!(
        screen.query_by_role(Role::TextBox, "Gain").is_some(),
        "a press opens the text, a double click does not reset"
    );
    screen.press_key(Key::Escape);
    gain.a11y(A11yAction::Increment);
    gain.a11y(A11yAction::Decrement);
    assert_eq!(value.get_untracked(), 10.0, "no relative write");
    assert!(log.borrow().is_empty(), "{:?}", log.borrow());

    let gain = screen.get_by_role(Role::SpinButton, "Gain");
    gain.press_key(Key::Enter);
    gain.focus();
    assert!(
        screen.query_by_role(Role::TextBox, "Gain").is_none(),
        "leaving the field closes it"
    );
    assert_eq!(
        value.get_untracked(),
        10.0,
        "an untouched text writes nothing"
    );
    assert!(log.borrow().is_empty(), "{:?}", log.borrow());
    gain.press_key(Key::Enter);
    let field = screen.get_by_role(Role::TextBox, "Gain");
    field.type_text("42");
    screen.press_key(Key::Enter);
    assert_eq!(value.get_untracked(), 42.0, "typed text writes");
    assert_eq!(*log.borrow(), [(Begin, 10.0), (Commit, 42.0)]);

    log.borrow_mut().clear();
    value.set(10.0);
    screen
        .get_by_role(Role::SpinButton, "Gain")
        .a11y_with_value(A11yAction::SetValue, "77");
    assert_eq!(value.get_untracked(), 77.0, "SetValue writes");
    assert_eq!(*log.borrow(), [(Begin, 10.0), (Commit, 77.0)]);

    mixed.set(false);
    screen.run_until_idle();
    let gain = screen.get_by_role(Role::SpinButton, "Gain");
    assert!(!gain.is_indeterminate());
    assert_eq!(gain.value().as_deref(), Some("77.000"));
    gain.press_key(Key::ArrowRight);
    assert_eq!(value.get_untracked(), 78.0, "a common value steps again");
    clean(&screen);
}

#[test]
fn the_mixed_text_is_a_message() {
    let screen = render_with(
        RenderOptions {
            theme: ui::theme(kivact::style::ThemeMode::Dark),
            messages: Messages::new().with("creative.numeric.mixed", "混在"),
            ..RenderOptions::from_env()
        },
        || ui::number("Gain", signal(0.0)).mixed(true),
    );
    assert!(screen.get_by_text("混在").exists());
    assert!(screen.query_by_text("Mixed").is_none());
    clean(&screen);
}

#[test]
fn the_property_state_follows_a_reactive_value_without_rebuilding() {
    let (screen, tint) = kivact::test::render_with_handle(crate::options(), || {
        let tint = signal(ui::PropertyState::Plain);
        let value = signal(1.0);
        (
            tint,
            column((
                ui::number("Scale", value).property_state(tint),
                ui::number("Fixed", signal(1.0)).property_state(ui::PropertyState::Plain),
            )),
        )
    });
    let scale = screen.get_by_role(Role::SpinButton, "Scale");
    scale.focus();
    let plain = scale.screenshot().hash();
    tint.set(ui::PropertyState::Keyed);
    screen.run_until_idle();
    let keyed = screen.get_by_role(Role::SpinButton, "Scale");
    assert!(keyed.is_focused(), "the field is not rebuilt");
    assert_ne!(keyed.screenshot().hash(), plain, "the tint changes");
    tint.set(ui::PropertyState::Plain);
    screen.run_until_idle();
    assert_eq!(scale.screenshot().hash(), plain);
    clean(&screen);
}

#[test]
fn the_caption_ellipsizes_and_the_value_keeps_its_width() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(), || {
        (
            (),
            ui::number("A very long property caption that cannot fit", signal(12.5))
                .range(0.0, 100.0)
                .precision(1)
                .style(style().width(120)),
        )
    });
    let value = screen.get_by_text("12.5");
    let rect = value.rect().expect("laid out");
    let field = screen
        .get_by_role(
            Role::SpinButton,
            "A very long property caption that cannot fit",
        )
        .rect()
        .expect("laid out");
    assert!(
        rect.x >= field.x && rect.x + rect.width <= field.x + field.width,
        "the value stays inside the field: {rect:?} in {field:?}"
    );
    let caption = screen.get_by_text("A very long property caption that cannot fit");
    assert!(caption.rect().expect("laid out").width < 120.0);
    clean(&screen);
}

#[test]
fn disposing_the_field_during_a_scrub_cancels_it() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, (value, present)) =
        kivact::test::render_with_handle(crate::options(), move || {
            let value = signal(10.0);
            let present = signal(true);
            (
                (value, present),
                show(present, move || {
                    ui::number("Gain", value)
                        .range(0.0, 100.0)
                        .step(1.0)
                        .on_edit(observer(value, &seen))
                }),
            )
        });
    let (x, y) = center(&screen, Role::SpinButton, "Gain");
    screen.mouse_move(x, y);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(x + 20.0, y);
    assert_eq!(value.get_untracked(), 30.0);
    present.set(false);
    screen.run_until_idle();
    assert_eq!(*log.borrow(), [(Begin, 10.0), (Cancel, 10.0)]);
    assert_eq!(value.get_untracked(), 10.0);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(log.borrow().len(), 2);
    clean(&screen);
}
