//! Role: Tests preserve shortcut letters across keyboard layouts.

use std::cell::RefCell;
use std::rc::Rc;

use kivact::interaction::use_text_edit;
use kivact::prelude::*;
use kivact::test::{Screen, render};
use kivact::view::KeyEvent;
use kivact::view::{KeyCode, Modifiers};

/// Press the key `physical` (a US-layout letter) that the layout makes
/// `layout`, with Ctrl held.
fn ctrl_on_layout(screen: &Screen, layout: char, physical: char) {
    let ctrl = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    let code = match physical {
        'a' => KeyCode::KeyA,
        'b' => KeyCode::KeyB,
        'c' => KeyCode::KeyC,
        'd' => KeyCode::KeyD,
        'e' => KeyCode::KeyE,
        'f' => KeyCode::KeyF,
        'g' => KeyCode::KeyG,
        'h' => KeyCode::KeyH,
        'i' => KeyCode::KeyI,
        'j' => KeyCode::KeyJ,
        'k' => KeyCode::KeyK,
        'l' => KeyCode::KeyL,
        'm' => KeyCode::KeyM,
        'n' => KeyCode::KeyN,
        'o' => KeyCode::KeyO,
        'p' => KeyCode::KeyP,
        'q' => KeyCode::KeyQ,
        'r' => KeyCode::KeyR,
        's' => KeyCode::KeyS,
        't' => KeyCode::KeyT,
        'u' => KeyCode::KeyU,
        'v' => KeyCode::KeyV,
        'w' => KeyCode::KeyW,
        'x' => KeyCode::KeyX,
        'y' => KeyCode::KeyY,
        'z' => KeyCode::KeyZ,
        _ => KeyCode::Unidentified,
    };
    screen.key_down(Key::Char(layout), code, ctrl);
    screen.key_up(Key::Char(layout), code, Modifiers::default());
}

#[test]
fn editing_chords_and_shortcuts_fall_back_to_the_physical_key_on_non_latin_layouts() {
    let keys: Rc<RefCell<Vec<String>>> = Rc::default();
    let log = keys.clone();
    let screen = render(move || {
        let value = signal("привет".to_string());
        node()
            .on_key(move |e: KeyEvent| {
                log.borrow_mut()
                    .push(format!("{:?} ctrl={}", e.key, e.modifiers.ctrl));
            })
            .child(node().name("Name").part(use_text_edit(value).control()))
    });
    screen.get_by_role(Role::TextBox, "Name").focus();
    // Russian layout: Ctrl+Ф is Ctrl+A, Ctrl+С is Ctrl+C.
    ctrl_on_layout(&screen, 'ф', 'a');
    ctrl_on_layout(&screen, 'с', 'c');
    assert_eq!(screen.clipboard().as_deref(), Some("привет"));
    // Ctrl+Ч is Ctrl+X: the field is cut.
    ctrl_on_layout(&screen, 'ч', 'x');
    assert_eq!(
        screen.get_by_role(Role::TextBox, "Name").value().as_deref(),
        Some("")
    );
    // An app shortcut (Ctrl+S) sees the Latin letter.
    ctrl_on_layout(&screen, 'ы', 's');
    // AZERTY: the physical Q key is the layout's A, and stays A.
    ctrl_on_layout(&screen, 'a', 'q');
    assert_eq!(
        keys.borrow().as_slice(),
        ["Char('s') ctrl=true", "Char('a') ctrl=true"],
        "Ctrl+A on an empty field is a no-op and reaches the handler as the layout's A"
    );
    screen.set_clipboard("abc");
    ctrl_on_layout(&screen, 'м', 'v');
    assert_eq!(
        screen.get_by_role(Role::TextBox, "Name").value().as_deref(),
        Some("abc")
    );
}
