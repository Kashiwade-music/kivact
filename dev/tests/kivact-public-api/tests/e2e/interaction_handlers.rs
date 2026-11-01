//! Role: Handler composition and raw inputs through real host windows.

use kivact::primitive::NodeExt;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use kivact::binding;
use kivact::prelude::*;
use kivact::test::render;
use kivact::view::PointerPhase;
use kivact::view::{A11yAction, Handled, Placement, PointerEvent, overlay};

type Log = Rc<RefCell<Vec<&'static str>>>;

// These two regressions use only APIs present on the baseline main.
#[test]
fn no_key_customization_preserves_submit_and_existing_handlers() {
    let log: Log = Rc::default();
    let output = log.clone();
    let screen = render(move || {
        let (old, new, submit) = (output.clone(), output.clone(), output.clone());
        kivact::control::node()
            .name("search")
            .part(
                kivact::interaction::use_text_edit(kivact::signal(String::new()))
                    .on_submit(move || submit.borrow_mut().push("submit"))
                    .control(),
            )
            .on_key(move |_| {
                old.borrow_mut().push("old");
                Handled::No
            })
            .on_key(move |_| {
                new.borrow_mut().push("new");
                Handled::No
            })
    });
    screen
        .get_by_role(Role::TextBox, "search")
        .press_key(Key::Enter);
    assert_eq!(*log.borrow(), ["new", "old", "submit"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn no_pointer_customization_preserves_existing_handler() {
    let log: Log = Rc::default();
    let output = log.clone();
    let screen = render(move || {
        let (old, new) = (output.clone(), output.clone());
        node()
            .test_id("hit")
            .style(style().size(80, 40))
            .on_pointer(move |e: PointerEvent| {
                if e.phase == PointerPhase::Down {
                    old.borrow_mut().push("old");
                }
                Handled::No
            })
            .on_pointer(move |e: PointerEvent| {
                if e.phase == PointerPhase::Down {
                    new.borrow_mut().push("new");
                }
                Handled::No
            })
    });
    screen.get_by_test_id("hit").click();
    assert_eq!(*log.borrow(), ["new", "old"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn consume_and_error_stop_the_right_chain() {
    let log: Log = Rc::default();
    let output = log.clone();
    let screen = render(move || {
        let (consume, unused, error, survivor, default) = (
            output.clone(),
            output.clone(),
            output.clone(),
            output.clone(),
            output.clone(),
        );
        column((
            kivact::control::node()
                .name("consume")
                .part(
                    kivact::interaction::use_text_edit(kivact::signal(String::new()))
                        .on_submit(move || unused.borrow_mut().push("unexpected submit"))
                        .control(),
                )
                .on_key(move |_| {
                    consume.borrow_mut().push("consume");
                    Handled::Yes
                }),
            kivact::control::node()
                .name("error")
                .part(
                    kivact::interaction::use_text_edit(kivact::signal(String::new()))
                        .on_submit(|| -> () { panic!("an error must stop the default") })
                        .control(),
                )
                .on_key(move |_| {
                    error.borrow_mut().push("error");
                    Err::<Handled, _>(UiError::msg("handler failed"))
                }),
            node()
                .role(Role::Button)
                .name("replacement")
                .focusable(true)
                .style(style().size(100, 30))
                .on_key(move |_| {
                    survivor.borrow_mut().push("replacement");
                    Handled::No
                })
                .on_key_default(move |_| {
                    default.borrow_mut().push("default");
                    Handled::Yes
                }),
        ))
    });
    screen
        .get_by_role(Role::TextBox, "consume")
        .press_key(Key::Enter);
    screen
        .get_by_role(Role::TextBox, "error")
        .press_key(Key::Enter);
    screen
        .get_by_role(Role::Button, "replacement")
        .press_key(Key::ArrowRight);
    assert_eq!(
        *log.borrow(),
        ["consume", "error", "replacement", "default"]
    );
    assert_eq!(
        screen.errors().len(),
        1,
        "the dispatch boundary reports once"
    );
    assert!(format!("{:?}", screen.errors()).contains("handler failed"));
}

#[test]
fn composed_press_runs_once_for_pointer_keyboard_and_accessibility() {
    let log: Log = Rc::default();
    let output = log.clone();
    let screen = render(move || {
        let (first, second, stale, final_action) = (
            output.clone(),
            output.clone(),
            output.clone(),
            output.clone(),
        );
        column((
            node()
                .role(Role::Button)
                .name("composed")
                .focusable(true)
                .style(style().size(100, 30))
                .on_press(move || {
                    first.borrow_mut().push("first");
                    second.borrow_mut().push("second");
                }),
            node()
                .role(Role::Button)
                .name("setter")
                .focusable(true)
                .style(style().size(100, 30))
                .on_press(move || stale.borrow_mut().push("stale"))
                .on_press(move || final_action.borrow_mut().push("setter")),
        ))
    });
    let button = screen.get_by_role(Role::Button, "composed");
    button.click();
    button.press_key(Key::Enter);
    button.press_key(Key::Space);
    button.a11y(A11yAction::Activate);
    assert_eq!(
        *log.borrow(),
        [
            "first", "second", "first", "second", "first", "second", "first", "second"
        ]
    );
    screen
        .get_by_role(Role::Button, "setter")
        .a11y(A11yAction::Activate);
    assert_eq!(log.borrow().last(), Some(&"setter"));
    assert!(!log.borrow().contains(&"stale"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn invoke_on_a11y_is_located_and_ignored_without_replacing_press() {
    let presses = Rc::new(Cell::new(0));
    let rejected = Rc::new(Cell::new(0));
    let at = Rc::new(Cell::new(0));
    let (count, bad, location) = (presses.clone(), rejected.clone(), at.clone());
    let screen = render(move || {
        let (count, bad) = (count.clone(), bad.clone());
        let button = node()
            .role(Role::Button)
            .name("preserved")
            .focusable(true)
            .style(style().size(100, 30))
            .on_press(move || count.set(count.get() + 1));
        location.set(line!() + 1);
        button.on_a11y(A11yAction::Activate, move || bad.set(bad.get() + 1))
    });
    let errors = screen.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(
        *errors[0].kind(),
        kivact::diagnostics::ErrorKind::Unsupported
    );
    assert!(errors[0].message().contains("on_press"));
    let location = errors[0].location().expect("located misuse");
    assert_eq!(location.file(), file!());
    assert_eq!(location.line(), at.get());
    let button = screen.get_by_role(Role::Button, "preserved");
    button.click();
    assert_eq!(presses.get(), 1);
    button.press_key(Key::Enter);
    assert_eq!(presses.get(), 2);
    button.press_key(Key::Space);
    assert_eq!(presses.get(), 3);
    button.a11y(A11yAction::Activate);
    assert_eq!(presses.get(), 4);
    assert_eq!(rejected.get(), 0);
    assert_eq!(screen.errors().len(), 1);
}

#[test]
fn a_press_only_node_is_reported_once_at_mount_in_a_debug_build() {
    let line = Rc::new(Cell::new(0));
    let at = line.clone();
    let screen = render(move || {
        at.set(line!() + 1);
        node()
            .test_id("pointer-only")
            .style(style().size(80, 30))
            .on_press(|| {})
    });
    let errors = screen.errors();
    if cfg!(debug_assertions) {
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(
            *errors[0].kind(),
            kivact::diagnostics::ErrorKind::Unsupported
        );
        assert!(errors[0].message().contains("on_press"));
        let location = errors[0].location().expect("located at the node");
        assert_eq!(location.file(), file!());
        assert_eq!(location.line(), line.get());
    } else {
        assert!(errors.is_empty(), "{errors:?}");
    }
    // The node is mounted and pressable by a pointer all the same.
    screen.get_by_test_id("pointer-only").click();
    assert_eq!(screen.errors().len(), errors.len());
}

#[test]
fn a_role_or_a_focus_stop_makes_a_pressable_node_reachable() {
    let presses = Rc::new(Cell::new(0));
    let count = presses.clone();
    let screen = render(move || {
        let (a, b, c) = (count.clone(), count.clone(), count.clone());
        (
            node()
                .role(Role::Button)
                .name("role")
                .style(style().size(80, 30))
                .on_press(move || a.set(a.get() + 1)),
            node()
                .focusable(true)
                .name("focus")
                .style(style().size(80, 30))
                .on_press(move || b.set(b.get() + 1)),
            // A deliberate pointer-only forwarder says so with `Role::Generic`.
            node()
                .role(Role::Generic)
                .test_id("forwarder")
                .style(style().size(80, 30))
                .on_press(move || c.set(c.get() + 1)),
        )
    });
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.get_by_test_id("forwarder").click();
    assert_eq!(presses.get(), 1);
}

#[test]
fn wheel_customizations_continue_to_default_and_native_scroll() {
    let log: Log = Rc::default();
    let output = log.clone();
    let screen = render(move || {
        let (old, new, default) = (output.clone(), output.clone(), output.clone());
        scroll(column((
            node()
                .test_id("wheel")
                .style(style().size(100, 40).shrink(0))
                .on_wheel(move |_| {
                    old.borrow_mut().push("old");
                    Handled::No
                })
                .on_wheel_default(move |_| {
                    default.borrow_mut().push("default");
                    Handled::No
                })
                .on_wheel(move |_| {
                    new.borrow_mut().push("new");
                    Handled::No
                }),
            node().style(style().size(100, 400).shrink(0)),
        )))
        .style(style().size(120, 100))
    });
    let target = screen.get_by_test_id("wheel");
    let before = target.rect().expect("before wheel");
    target.scroll(0.0, 24.0);
    let after = target.rect().expect("after wheel");
    assert_eq!(*log.borrow(), ["new", "old", "default"]);
    assert!(
        after.y < before.y,
        "native scrolling: {before:?} -> {after:?}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn native_and_styled_editors_share_controlled_composition_and_submit() {
    let (tx, rx) = std::sync::mpsc::channel();
    let screen = render(move || {
        let value = signal(String::from("start"));
        let submits = signal(0);
        tx.send((value, submits)).expect("test receiver alive");
        column((
            kivact::control::node()
                .name("raw")
                .part(
                    kivact::interaction::use_text_edit(value)
                        .on_submit(move || submits.update(|n| *n += 1))
                        .control(),
                )
                .style(style().width(150).height(35))
                .on_key(|_| Handled::No),
            kivact::control::node()
                .name("recipe")
                .part(
                    kivact::interaction::use_text_edit(kivact::signal(String::new()))
                        .multiline(true)
                        .control(),
                )
                .style(style().width(150).height(60)),
        ))
    });
    let (value, submits) = rx.recv().expect("mounted handles");
    let raw = screen.get_by_role(Role::TextBox, "raw");
    raw.press_key(Key::End);
    raw.compose("日本");
    assert_eq!(value.get_untracked(), "start", "preedit is not committed");
    raw.commit("日本");
    assert_eq!(value.get_untracked(), "start日本");
    raw.press_key(Key::Enter);
    assert_eq!(submits.get_untracked(), 1);
    value.set(String::from("external"));
    screen.run_until_idle();
    assert_eq!(raw.value().as_deref(), Some("external"));
    let area = screen.get_by_role(Role::TextBox, "recipe");
    area.type_text("line");
    area.press_key(Key::Enter);
    area.type_text("two");
    assert_eq!(area.value().as_deref(), Some("line\ntwo"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn failed_key_and_wheel_routes_stop_ancestors_and_client_defaults() {
    let log: Log = Rc::default();
    let output = log.clone();
    let screen = render(move || {
        let (parent_key, activate, dismiss, parent_wheel) = (
            output.clone(),
            output.clone(),
            output.clone(),
            output.clone(),
        );
        let anchor = node_ref();
        column((
            node()
                .on_key(move |_| {
                    parent_key.borrow_mut().push("parent key");
                    Handled::No
                })
                .child(
                    node()
                        .role(Role::Button)
                        .name("failed key")
                        .focusable(true)
                        .style(style().size(100, 30))
                        .on_press(move || activate.borrow_mut().push("activate"))
                        .on_key(|_| Err::<Handled, _>(UiError::msg("key route failed"))),
                ),
            overlay(
                anchor,
                Placement::Below,
                node()
                    .role(Role::Button)
                    .name("failed escape")
                    .focusable(true)
                    .style(style().size(100, 30))
                    .on_key(|_| Err::<Handled, _>(UiError::msg("escape route failed"))),
            )
            .role(Role::Menu)
            .name("stays open")
            .on_dismiss(move |_| dismiss.borrow_mut().push("dismiss")),
            scroll(column((
                node()
                    .test_id("failed wheel")
                    .style(style().size(100, 40).shrink(0))
                    .on_wheel(|_| Err::<Handled, _>(UiError::msg("wheel route failed"))),
                node().style(style().size(100, 400).shrink(0)),
            )))
            .style(style().size(120, 100))
            .on_wheel(move |_| {
                parent_wheel.borrow_mut().push("parent wheel");
                Handled::No
            }),
        ))
    });
    screen
        .get_by_role(Role::Button, "failed key")
        .press_key(Key::Enter);
    screen
        .get_by_role(Role::Button, "failed escape")
        .press_key(Key::Escape);
    let wheel = screen.get_by_test_id("failed wheel");
    let before = wheel.rect().expect("before wheel");
    wheel.scroll(0.0, 24.0);
    assert_eq!(
        wheel.rect(),
        Some(before),
        "failed wheel cannot scroll natively"
    );
    assert!(log.borrow().is_empty(), "{:?}", log.borrow());
    assert_eq!(
        screen.errors().len(),
        3,
        "each failed event is reported once"
    );
}

#[test]
fn fallible_bindings_report_once_and_keep_caller_values_authoritative() {
    use kivact::interaction::{ListGesture, ListPolicy, ListSelection, use_list, use_scalar};
    use kivact::ui::{ListItem, ScalarRange};

    let offers = Rc::new(Cell::new(0));
    let count = offers.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    let screen = render(move || {
        let scalar_value = signal(5.0);
        let choice_value = signal(Some("a"));
        let selected = signal(vec!["a"]);
        let scalar_count = count.clone();
        let choice_count = count.clone();
        let selection_count = count.clone();
        let slider = use_scalar(
            binding(scalar_value.into_value(), move |_| {
                scalar_count.set(scalar_count.get() + 1);
                Err::<(), _>(UiError::msg("scalar writer failed"))
            }),
            ScalarRange::new(0.0, 10.0).expect("finite ordered range"),
        );
        let choices = use_list(
            vec![ListItem::new("a", ""), ListItem::new("b", "")],
            ListSelection::Single(binding(choice_value.into_value(), move |_| {
                choice_count.set(choice_count.get() + 1);
                Err::<(), _>(UiError::msg("choice writer failed"))
            })),
        )
        .policy(ListPolicy::radio());
        let selection = use_list(
            (vec!["a", "b"])
                .into_iter()
                .map(|key| ListItem::new(key, ""))
                .collect::<Vec<_>>(),
            ListSelection::Multiple(binding(selected.into_value(), move |_| {
                selection_count.set(selection_count.get() + 1);
                Err::<(), _>(UiError::msg("selection writer failed"))
            })),
        );
        tx.send((scalar_value, choice_value, selected, choices, selection))
            .expect("test receiver is alive");
        column((
            node()
                .part(slider.thumb())
                .name("scalar")
                .style(style().size(160, 32)),
            node().part(choices.list().role(Role::TabList)).child((
                node()
                    .part(choices.item("a").role(Role::Tab).name("a"))
                    .child(text("a")),
                node()
                    .part(choices.item("b").role(Role::Tab).name("b"))
                    .child(text("b")),
            )),
            node()
                .role(Role::Button)
                .name("select b")
                .focusable(true)
                .style(style().size(100, 32))
                .child(text("select b"))
                .on_press(move || {
                    selection.select("b", ListGesture::Replace);
                }),
        ))
    });
    let (scalar_value, choice_value, selected, choices, selection) =
        rx.recv().expect("mounted handles");
    screen
        .get_by_role(Role::Slider, "scalar")
        .a11y(A11yAction::Increment);
    assert_eq!(scalar_value.get_untracked(), 5.0);
    assert_eq!(offers.get(), 1);
    screen
        .get_by_role(Role::Tab, "b")
        .a11y(A11yAction::Activate);
    assert_eq!(choice_value.get_untracked(), Some("a"));
    assert_eq!(choices.active().get_untracked(), Some("b"));
    assert!(screen.get_by_role(Role::Tab, "b").is_focused());
    assert_eq!(offers.get(), 2);
    screen.get_by_role(Role::Button, "select b").click();
    assert_eq!(selected.get_untracked(), vec!["a"]);
    assert_eq!(selection.anchor_value().get_untracked(), Some("b"));
    assert_eq!(offers.get(), 3);
    assert_eq!(
        screen.errors().len(),
        3,
        "one report for each rejected writer"
    );
}

#[test]
fn prevent_default_keys_bubble_without_activation_or_dismissal() {
    let log: Log = Rc::default();
    let output = log.clone();
    let presses = Rc::new(Cell::new(0));
    let count = presses.clone();
    let dismissals = Rc::new(Cell::new(0));
    let dismissed = dismissals.clone();
    let screen = render(move || {
        let (inner, outer) = (output.clone(), output.clone());
        let (count, dismissed) = (count.clone(), dismissed.clone());
        let anchor = node_ref();
        column((
            node().node_ref(anchor).style(style().size(10, 10)),
            overlay(
                anchor,
                Placement::Below,
                column((node()
                    .role(Role::Button)
                    .name("target")
                    .focusable(true)
                    .style(style().size(100, 30))
                    .on_press(move || count.set(count.get() + 1))
                    .on_key_default(|_| Handled::No)
                    .on_key(|_| Handled::PreventDefault),))
                .on_key(move |event| {
                    inner.borrow_mut().push("inner");
                    if event.key == Key::ArrowDown {
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                }),
            )
            .role(Role::Dialog)
            .name("Layer")
            .on_dismiss(move |_| dismissed.set(dismissed.get() + 1)),
        ))
        .on_key(move |_| {
            outer.borrow_mut().push("outer");
            Handled::No
        })
    });
    let target = screen.get_by_role(Role::Button, "target");
    target.press_key(Key::Space);
    target.press_key(Key::Enter);
    assert_eq!(presses.get(), 0, "a prevented key does not activate");
    assert_eq!(*log.borrow(), ["inner", "outer", "inner", "outer"]);
    target.press_key(Key::Escape);
    assert_eq!(dismissals.get(), 0, "a prevented Escape does not dismiss");
    log.borrow_mut().clear();
    target.press_key(Key::ArrowDown);
    assert_eq!(
        *log.borrow(),
        ["inner"],
        "a later Yes still stops the bubbling"
    );
    target.click();
    assert_eq!(presses.get(), 1, "the pointer activation is unaffected");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn prevent_default_wheel_bubbles_without_native_scroll() {
    let log: Log = Rc::default();
    let output = log.clone();
    let screen = render(move || {
        let (inner, outer) = (output.clone(), output.clone());
        scroll(
            column((
                node()
                    .test_id("wheel")
                    .style(style().size(100, 40).shrink(0))
                    .on_wheel_default(|_| Handled::No)
                    .on_wheel(move |_| {
                        inner.borrow_mut().push("inner");
                        Handled::PreventDefault
                    }),
                node().style(style().size(100, 400).shrink(0)),
            ))
            .on_wheel(move |_| {
                outer.borrow_mut().push("outer");
                Handled::No
            }),
        )
        .style(style().size(120, 100))
    });
    let target = screen.get_by_test_id("wheel");
    let before = target.rect().expect("before wheel");
    target.scroll(0.0, 24.0);
    assert_eq!(*log.borrow(), ["inner", "outer"], "the wheel still bubbles");
    assert_eq!(
        target.rect().expect("after wheel"),
        before,
        "the prevented wheel does not scroll"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
