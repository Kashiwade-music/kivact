//! Role: Input handlers run in a fixed layer order with parts and regions.
use super::*;
use kivact::primitive::NodeExt;

type Log = Rc<RefCell<Vec<&'static str>>>;

fn log_phase_down(log: &Log, name: &'static str, event: &kivact::view::PointerEvent) {
    if event.phase == PointerPhase::Down {
        log.borrow_mut().push(name);
    }
}

#[test]
fn callers_run_last_registered_first_before_node_and_part_defaults() {
    let log: Log = Rc::default();
    let out = log.clone();
    let (screen, ()) = mount(move || {
        let (part_log, default_log, first, second) =
            (out.clone(), out.clone(), out.clone(), out.clone());
        let behavior = part(|state| state)
            .role(Role::Button)
            .name("Layered")
            .focusable(true)
            .on_key(move |_| {
                part_log.borrow_mut().push("part default");
                Handled::No
            });
        // The node's default is registered after the part is attached and
        // still runs before the part's default.
        let control = node()
            .part(behavior)
            .on_key_default(move |_| {
                default_log.borrow_mut().push("node default");
                Handled::No
            })
            .on_key(move |_| {
                first.borrow_mut().push("first");
                Handled::No
            })
            .on_key(move |_| {
                second.borrow_mut().push("second");
                Handled::No
            })
            .style(style().size(100, 30));
        ((), control)
    });
    screen
        .get_by_role(Role::Button, "Layered")
        .press_key(Key::ArrowRight);
    assert_eq!(
        *log.borrow(),
        ["second", "first", "node default", "part default"]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_consuming_caller_skips_part_defaults_but_not_the_part_state_observer() {
    let log: Log = Rc::default();
    let out = log.clone();
    let (screen, state) = mount(move || {
        let (part_log, caller_log) = (out.clone(), out.clone());
        let behavior = part(|state| state)
            .role(Role::Button)
            .name("Consumed")
            .focusable(true)
            .on_key(move |_| {
                part_log.borrow_mut().push("part default");
                Handled::No
            });
        let state = behavior.state();
        let control = node()
            .part(behavior)
            .on_key(move |_| {
                caller_log.borrow_mut().push("caller");
                Handled::Yes
            })
            .style(style().size(100, 30));
        (state, control)
    });
    screen.get_by_role(Role::Button, "Consumed").focus();
    screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    assert!(state.get().pressed, "the observer runs before the caller");
    assert_eq!(*log.borrow(), ["caller"]);
    screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
    assert!(!state.get().pressed);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_prevented_default_still_reaches_part_defaults_but_not_the_default_action() {
    for (answer, activates) in [(Handled::PreventDefault, false), (Handled::No, true)] {
        let log: Log = Rc::default();
        let out = log.clone();
        let (screen, ()) = mount(move || {
            let (part_log, caller_log, press_log) = (out.clone(), out.clone(), out.clone());
            let behavior = part(|state| state)
                .role(Role::Button)
                .name("Prevented")
                .focusable(true)
                .on_key(move |_| {
                    part_log.borrow_mut().push("part default");
                    Handled::No
                });
            let control = node()
                .part(behavior)
                .on_key(move |_| {
                    caller_log.borrow_mut().push("caller");
                    answer
                })
                .on_press(move || press_log.borrow_mut().push("press"))
                .style(style().size(100, 30));
            ((), control)
        });
        screen
            .get_by_role(Role::Button, "Prevented")
            .press_key(Key::Enter);
        let expected: &[&str] = if activates {
            &["caller", "part default", "press"]
        } else {
            &["caller", "part default"]
        };
        assert_eq!(*log.borrow(), expected, "{answer:?}");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[test]
fn a_region_key_goes_to_the_region_default_then_the_node_fallback() {
    let log: Log = Rc::default();
    let out = log.clone();
    let (screen, ()) = mount(move || {
        let (caller_log, region_log, fallback_log) = (out.clone(), out.clone(), out.clone());
        let fallback = part(|state| state).region_events().on_key(move |_| {
            fallback_log.borrow_mut().push("fallback");
            Handled::No
        });
        let root = canvas(|draw, _| {
            draw.region(RegionId(1), Rect::new(0., 0., 80., 40.));
        })
        .name("Canvas")
        .role(Role::Canvas)
        .style(style().size(120, 80))
        .part(fallback)
        .region(
            RegionId(1),
            part(|state| state)
                .name("Region")
                .role(Role::Button)
                .focusable(true)
                .on_key(move |event| {
                    region_log.borrow_mut().push("region");
                    if event.key == Key::Char('h') {
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                }),
        )
        .on_key(move |_| {
            caller_log.borrow_mut().push("caller");
            Handled::No
        });
        ((), root)
    });
    screen.get_by_role(Role::Button, "Region").focus();
    screen.press_key(Key::ArrowRight);
    assert_eq!(*log.borrow(), ["caller", "region", "fallback"]);
    log.borrow_mut().clear();
    screen.press_key(Key::Char('h'));
    assert_eq!(
        *log.borrow(),
        ["caller", "region"],
        "a consumed key does not reach the fallback"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_node_default_sees_region_pointer_input_before_the_part_defaults() {
    let log: Log = Rc::default();
    let out = log.clone();
    let (screen, ()) = mount(move || {
        let (caller_log, default_log, region_log, fallback_log) =
            (out.clone(), out.clone(), out.clone(), out.clone());
        let fallback = part(|state| state)
            .region_events()
            .on_pointer(move |event| {
                log_phase_down(&fallback_log, "fallback", &event);
                Handled::No
            });
        let root = canvas(|draw, _| {
            draw.region(RegionId(1), Rect::new(0., 0., 80., 40.));
        })
        .name("Canvas")
        .role(Role::Canvas)
        .style(style().size(120, 80))
        .part(fallback)
        .region(
            RegionId(1),
            part(|state| state)
                .name("Region")
                .role(Role::Button)
                .focusable(true)
                .on_pointer(move |event| {
                    log_phase_down(&region_log, "region", &event);
                    Handled::No
                }),
        )
        .on_pointer_default(move |event| {
            log_phase_down(&default_log, "node default", &event);
            Handled::No
        })
        .on_pointer(move |event| {
            log_phase_down(&caller_log, "caller", &event);
            Handled::No
        });
        ((), root)
    });
    screen.mouse_move(20., 20.);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(
        *log.borrow(),
        ["caller", "node default", "region", "fallback"]
    );
    log.borrow_mut().clear();
    // Outside every region the node's defaults still run, and the opted part's
    // handler is an ordinary default of the node.
    screen.mouse_move(100., 60.);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(*log.borrow(), ["caller", "node default", "fallback"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_node_pointer_default_runs_before_the_part_defaults_whatever_the_call_order() {
    let log: Log = Rc::default();
    let out = log.clone();
    let (screen, ()) = mount(move || {
        let (part_log, default_log) = (out.clone(), out.clone());
        let behavior = part(|state| state).on_pointer(move |event| {
            log_phase_down(&part_log, "part default", &event);
            Handled::No
        });
        let control = node()
            .test_id("hit")
            .part(behavior)
            .on_pointer_default(move |event| {
                log_phase_down(&default_log, "node default", &event);
                Handled::No
            })
            .style(style().size(80, 40));
        ((), control)
    });
    screen.get_by_test_id("hit").click();
    assert_eq!(*log.borrow(), ["node default", "part default"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn hover_transitions_between_regions_are_delivered_one_by_one_even_when_consumed() {
    let log: Rc<RefCell<Vec<(u64, PointerPhase)>>> = Rc::default();
    let out = log.clone();
    let (screen, ()) = mount(move || {
        let region = |id: u64| {
            let log = out.clone();
            part(|state| state).on_pointer(move |event| {
                log.borrow_mut().push((id, event.phase));
                Handled::Yes
            })
        };
        let root = canvas(|draw, _| {
            draw.region(RegionId(1), Rect::new(0., 0., 40., 40.));
            draw.region(RegionId(2), Rect::new(60., 0., 40., 40.));
        })
        .name("Canvas")
        .role(Role::Canvas)
        .style(style().size(120, 80))
        .region(RegionId(1), region(1))
        .region(RegionId(2), region(2));
        ((), root)
    });
    screen.mouse_move(20., 20.);
    log.borrow_mut().clear();
    screen.mouse_move(80., 20.);
    // A consumed leave does not stop the enter or the move that caused it.
    assert_eq!(
        *log.borrow(),
        [
            (1, PointerPhase::Leave),
            (2, PointerPhase::Enter),
            (2, PointerPhase::Move)
        ]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
