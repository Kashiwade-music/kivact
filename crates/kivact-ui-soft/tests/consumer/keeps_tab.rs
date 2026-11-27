//! Role: Verify a focusable node can keep Tab for its own key handlers.
use std::cell::RefCell;
use std::rc::Rc;

use kivact::state::signal;
use kivact::style::ThemeMode;
use kivact::test::MirrorMode;
use kivact::test::RenderOptions;
use kivact::test::Screen;
use kivact::test::render_with_handle;
use kivact::view::Key;
use kivact::view::Modifiers;
use kivact::view::Node;
use kivact::view::Role;
use kivact::view::column;
use kivact::view::node;

/// The keys the middle control's handler received, with their modifiers.
type Log = Rc<RefCell<Vec<(Key, Modifiers)>>>;

fn options() -> RenderOptions {
    RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)).mirror(MirrorMode::Off)
}

fn modifiers(shift: bool, ctrl: bool, alt: bool, meta: bool) -> Modifiers {
    Modifiers {
        shift,
        ctrl,
        alt,
        meta,
    }
}

/// Whether the focus is on the element named `name`.
fn focus_is(screen: &Screen, name: &str) -> bool {
    screen.focused().is_some_and(|focus| focus.name() == name)
}

/// A plain focusable control that records the keys its handler receives.
fn keeping_canvas(log: &Log, keeps: bool) -> Node {
    let sink = Rc::clone(log);
    node()
        .name("Canvas")
        .role(Role::Canvas)
        .focusable(true)
        .keeps_tab(keeps)
        .on_key(move |event| sink.borrow_mut().push((event.key(), event.modifiers())))
}

/// Three focusable stops in order: `Before`, the middle control, `After`.
fn stops(log: &Log, keeps: bool) -> impl FnOnce() -> ((), Node) + 'static {
    let log = Rc::clone(log);
    move || {
        (
            (),
            column((
                node().name("Before").focusable(true),
                keeping_canvas(&log, keeps),
                node().name("After").focusable(true),
            )),
        )
    }
}

#[test]
fn tab_reaches_a_node_that_keeps_tab() {
    let log: Log = Rc::default();
    let (screen, ()) = render_with_handle(options(), stops(&log, true));
    let canvas = screen.get_by_role(Role::Canvas, "Canvas");

    canvas.focus();
    screen.press_key(Key::Tab);
    screen.press_key_with(Key::Tab, modifiers(true, false, false, false));

    assert_eq!(
        *log.borrow(),
        vec![
            (Key::Tab, modifiers(false, false, false, false)),
            (Key::Tab, modifiers(true, false, false, false)),
        ]
    );
    assert!(canvas.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn ctrl_tab_leaves_a_node_that_keeps_tab() {
    let log: Log = Rc::default();
    let (screen, ()) = render_with_handle(options(), stops(&log, true));
    let canvas = screen.get_by_role(Role::Canvas, "Canvas");

    canvas.focus();
    screen.press_key_with(Key::Tab, modifiers(false, true, false, false));
    assert!(focus_is(&screen, "After"));
    assert!(log.borrow().is_empty());

    canvas.focus();
    screen.press_key_with(Key::Tab, modifiers(true, true, false, false));
    assert!(focus_is(&screen, "Before"));
    assert!(log.borrow().is_empty());

    canvas.focus();
    screen.press_key_with(Key::Tab, modifiers(false, true, false, true));
    assert!(focus_is(&screen, "After"));
    assert!(log.borrow().is_empty());

    canvas.focus();
    screen.press_key_with(Key::Tab, modifiers(false, true, true, false));
    assert!(canvas.is_focused());
    assert_eq!(
        *log.borrow(),
        vec![(Key::Tab, modifiers(false, true, true, false))]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn tab_still_moves_focus_from_other_nodes() {
    let log: Log = Rc::default();
    let (screen, ()) = render_with_handle(options(), stops(&log, false));

    screen.press_key(Key::Tab);
    assert!(focus_is(&screen, "Before"));
    screen.press_key(Key::Tab);
    assert!(focus_is(&screen, "Canvas"));
    screen.press_key(Key::Tab);
    assert!(focus_is(&screen, "After"));
    screen.press_key_with(Key::Tab, modifiers(true, false, false, false));
    assert!(focus_is(&screen, "Canvas"));
    assert!(log.borrow().is_empty());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn keeps_tab_can_be_switched_off() {
    let log: Log = Rc::default();
    let sink = Rc::clone(&log);
    let (screen, keeps) = render_with_handle(options(), move || {
        let keeps = signal(true);
        let sink = Rc::clone(&sink);
        (
            keeps,
            column((
                node().name("Before").focusable(true),
                node()
                    .name("Canvas")
                    .role(Role::Canvas)
                    .focusable(true)
                    .keeps_tab(move || keeps.get())
                    .on_key(move |event| sink.borrow_mut().push((event.key(), event.modifiers()))),
                node().name("After").focusable(true),
            )),
        )
    });
    let canvas = screen.get_by_role(Role::Canvas, "Canvas");

    canvas.focus();
    screen.press_key(Key::Tab);
    assert_eq!(
        *log.borrow(),
        vec![(Key::Tab, modifiers(false, false, false, false))]
    );
    assert!(canvas.is_focused());

    keeps.set(false);
    screen.run_until_idle();
    screen.press_key(Key::Tab);
    assert!(focus_is(&screen, "After"));
    assert_eq!(log.borrow().len(), 1);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
