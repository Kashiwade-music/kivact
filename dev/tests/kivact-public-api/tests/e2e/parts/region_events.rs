//! Role: Opted-in canvas behavior receives only unhandled region input.
use super::*;
use kivact::interaction::{ViewTransform, use_zoom_pan};
use kivact::test::Screen;
use kivact::ui::EditPhase;

#[test]
fn region_input_preserves_customization_identity_and_other_parts_isolation() {
    let (screen, (calls, unrelated, block, caller_block)) = mount(|| {
        let calls = signal(Vec::<(&'static str, Option<RegionId>)>::new());
        let unrelated = signal(0);
        let block = signal(false);
        let caller_block = signal(false);
        let opted = part(|state| state)
            .region_events()
            .on_pointer(move |event| {
                if event.phase == PointerPhase::Down {
                    calls.update(|calls| calls.push(("pointer", event.region())));
                }
                Handled::No
            })
            .on_key(move |event| {
                calls.update(|calls| calls.push(("key", event.region)));
                Handled::Yes
            })
            .on_wheel(move |event| {
                calls.update(|calls| calls.push(("wheel", event.region())));
                Handled::Yes
            });
        let ordinary = part(|state| state)
            .on_pointer(move |_| {
                unrelated.update(|value| *value += 1);
                Handled::No
            })
            .on_key(move |_| {
                unrelated.update(|value| *value += 1);
                Handled::No
            });
        let root = canvas(|draw, _| {
            draw.region(RegionId(1), Rect::new(0., 0., 80., 40.));
        })
        .name("Canvas")
        .role(Role::Canvas)
        .style(style().size(120, 80))
        .part(opted.with(ordinary))
        .region(
            RegionId(1),
            part(|state| state)
                .name("Region")
                .role(Role::Button)
                .focusable(true)
                .on_pointer(move |event| {
                    if block.get_untracked() && event.phase == PointerPhase::Down {
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                })
                .on_key(|event| {
                    if event.key == Key::Char('h') {
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                }),
        )
        .on_pointer(move |event| {
            if caller_block.get_untracked() && event.phase == PointerPhase::Down {
                Handled::Yes
            } else {
                Handled::No
            }
        });
        ((calls, unrelated, block, caller_block), root)
    });
    screen.mouse_move(20., 20.);
    screen.mouse_down(MouseButton::Middle);
    screen.mouse_up(MouseButton::Middle);
    screen.get_by_role(Role::Button, "Region").focus();
    screen.press_key(Key::ArrowRight);
    screen.scroll(0., -20.);
    assert_eq!(
        calls.get(),
        vec![
            ("pointer", Some(RegionId(1))),
            ("key", Some(RegionId(1))),
            ("wheel", Some(RegionId(1)))
        ]
    );
    assert_eq!(unrelated.get(), 0);
    screen.press_key(Key::Char('h'));
    block.set(true);
    screen.mouse_down(MouseButton::Middle);
    screen.mouse_up(MouseButton::Middle);
    block.set(false);
    caller_block.set(true);
    screen.mouse_down(MouseButton::Middle);
    screen.mouse_up(MouseButton::Middle);
    assert_eq!(calls.get().len(), 3);
    assert_eq!(unrelated.get(), 0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn region_pan_uses_shared_transform_and_cancels_when_its_target_retires() {
    let (screen, (transform, visible, phases)) = mount(|| {
        let transform = signal(ViewTransform::default());
        let visible = signal(true);
        let phases = signal(Vec::new());
        let root = canvas(move |draw, _| {
            if visible.get() {
                draw.region(RegionId(1), Rect::new(0., 0., 80., 40.));
            }
        })
        .name("Canvas")
        .role(Role::Canvas)
        .style(style().size(120, 80))
        .part(
            use_zoom_pan(transform)
                .on_edit(move |phase| phases.update(|phases| phases.push(phase)))
                .viewport(),
        )
        .regions(
            move || {
                if visible.get() {
                    vec![1_u64]
                } else {
                    Vec::new()
                }
            },
            |id| RegionId(*id),
            |_, _| {
                part(|state| state)
                    .role(Role::Group)
                    .name("Region")
                    .focusable(true)
            },
        );
        ((transform, visible, phases), root)
    });
    let region = screen.get_by_role(Role::Group, "Region");
    region.focus();
    screen.mouse_move(20., 20.);
    screen.mouse_down(MouseButton::Middle);
    screen.mouse_move(50., 30.);
    assert_eq!(transform.get().offset, Point::new(30., 10.));
    visible.set(false);
    screen.run_until_idle();
    assert_eq!(transform.get(), ViewTransform::default());
    screen.mouse_move(70., 35.);
    screen.mouse_up(MouseButton::Middle);
    assert_eq!(transform.get(), ViewTransform::default());
    assert_eq!(phases.get(), vec![EditPhase::Begin, EditPhase::Cancel]);
    visible.set(true);
    screen.run_until_idle();
    screen.get_by_role(Role::Group, "Region").focus();
    screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    screen.mouse_move(20., 20.);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(30., 35.);
    screen.mouse_up(MouseButton::Primary);
    screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
    assert_eq!(transform.get().offset, Point::new(10., 15.));
    screen.scroll(0., -20.);
    assert!(transform.get().scale > 1.);
    assert_eq!(
        phases
            .get()
            .iter()
            .filter(|p| **p == EditPhase::Commit)
            .count(),
        2
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn adopted_region_gestures_finish_across_disabled_hits_and_consumed_terminal_events() {
    for mode in 0..4 {
        let (screen, (transform, phases, identities)) = mount(move || {
            let transform = signal(ViewTransform::default());
            let phases = signal(Vec::new());
            let identities = signal(Vec::new());
            let root = canvas(|draw, _| {
                draw.region(RegionId(1), Rect::new(0., 0., 40., 40.));
                draw.region(RegionId(2), Rect::new(50., 0., 40., 40.));
            })
            .role(Role::Canvas)
            .style(style().size(120, 80))
            .part(
                part(|state| state)
                    .region_events()
                    .on_pointer(move |event| {
                        identities.update(|ids| ids.push(event.region()));
                        Handled::No
                    }),
            )
            .part(
                use_zoom_pan(transform)
                    .on_edit(move |phase| phases.update(|p| p.push(phase)))
                    .viewport(),
            )
            .region(
                RegionId(1),
                part(|state| state)
                    .role(Role::Group)
                    .name("Start")
                    .focusable(true)
                    .on_pointer(move |event| {
                        if mode == 3 && event.phase == PointerPhase::Cancel {
                            Handled::Yes
                        } else {
                            Handled::No
                        }
                    }),
            )
            .region(
                RegionId(2),
                part(|state| state)
                    .role(Role::Group)
                    .name("Disabled")
                    .disabled(true),
            )
            .on_pointer(move |event| {
                if (mode == 1 && event.phase == PointerPhase::Up)
                    || (mode == 2 && event.phase == PointerPhase::Cancel)
                {
                    Handled::Yes
                } else {
                    Handled::No
                }
            });
            ((transform, phases, identities), root)
        });
        screen.get_by_role(Role::Group, "Start").focus();
        screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
        let contact = screen.contact_down(kivact::view::PointerKind::Touch, 20., 20.);
        screen.contact_move(contact, 70., 30.);
        assert_eq!(transform.get().offset, Point::new(50., 10.));
        if mode < 2 {
            screen.contact_up(contact);
        } else {
            screen.contact_cancel(contact);
        }
        screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
        if mode == 0 {
            assert_eq!(phases.get(), vec![EditPhase::Begin, EditPhase::Commit]);
            assert_eq!(transform.get().offset, Point::new(50., 10.));
        } else {
            assert_eq!(phases.get(), vec![EditPhase::Begin, EditPhase::Cancel]);
            assert_eq!(transform.get(), ViewTransform::default());
        }
        assert!(!identities.get().is_empty());
        assert!(identities.get().iter().all(|id| *id == Some(RegionId(1))));
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[test]
fn a_region_pan_declines_its_pointer_click_without_suppressing_later_at_or_clicks() {
    let (screen, (transform, presses, spaces)) = mount(|| {
        let transform = signal(ViewTransform::default());
        let presses = signal(0);
        let spaces = signal(0);
        let root = canvas(|draw, _| {
            draw.region(RegionId(1), Rect::new(0., 0., 100., 60.));
        })
        .role(Role::Canvas)
        .style(style().size(120, 80))
        .part(use_zoom_pan(transform).viewport())
        .region(
            RegionId(1),
            part(|state| state)
                .role(Role::Button)
                .name("Action")
                .focusable(true)
                .on_press(move || presses.update(|value| *value += 1)),
        );
        let app = column((root,)).on_key(move |event| {
            if event.key == Key::Space {
                spaces.update(|value| *value += 1);
            }
            Handled::No
        });
        ((transform, presses, spaces), app)
    });
    let action = screen.get_by_role(Role::Button, "Action");
    action.focus();
    screen.key_down(Key::Space, KeyCode::Space, Modifiers::default());
    // Idle Space bubbles to the application but never activates the focused
    // control inside the pan surface.
    assert_eq!(spaces.get(), 1, "idle Space reaches the application");
    assert_eq!(presses.get(), 0, "idle Space does not press the region");
    screen.mouse_move(20., 20.);
    screen.mouse_down(MouseButton::Primary);
    screen.mouse_move(40., 30.);
    screen.mouse_down(MouseButton::Secondary);
    screen.mouse_up(MouseButton::Secondary);
    screen.key_up(Key::Space, KeyCode::Space, Modifiers::default());
    // A contact is a pointer of its own: its tap during the mouse's pan
    // clicks the region (the pan declines only its own pointer's click).
    let contact = screen.contact_down(kivact::view::PointerKind::Touch, 40., 30.);
    screen.contact_up(contact);
    assert_eq!(presses.get(), 1, "another pointer's tap during the pan");
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(transform.get().offset, Point::new(20., 10.));
    assert_eq!(presses.get(), 1, "a pan is not also a pointer click");
    action.a11y(A11yAction::Activate);
    assert_eq!(
        presses.get(),
        2,
        "AT activation does not inherit pointer suppression"
    );
    action.click();
    assert_eq!(
        presses.get(),
        3,
        "the next ordinary press has its normal activation"
    );
    action.press_key(Key::Enter);
    assert_eq!(presses.get(), 4, "Enter keeps its activation");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// What [`claiming_canvas`] hands out: the claim switch, the region's and the
/// parent's press counts and the pointer events the canvas saw.
type Claiming = (
    Signal<bool>,
    Signal<u32>,
    Signal<u32>,
    Signal<Vec<(PointerPhase, Option<RegionId>)>>,
);

/// A canvas whose region 1 is a button and whose fallback claims the press of
/// that region while `claim` is set, inside a button of its own.
fn claiming_canvas() -> (Screen, Claiming) {
    mount(|| {
        let claim = signal(true);
        let region = signal(0);
        let parent = signal(0);
        let seen = signal(Vec::new());
        let surface = canvas(|draw, _| {
            draw.region(RegionId(1), Rect::new(0., 0., 60., 40.));
        })
        .name("Surface")
        .role(Role::Canvas)
        .style(style().size(120, 80))
        .part(
            part(|state| state)
                .region_events()
                .on_pointer(move |event| {
                    if event.phase == PointerPhase::Down && claim.get_untracked() {
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                }),
        )
        .on_pointer(move |event| {
            seen.update(|seen| seen.push((event.phase, event.region())));
            Handled::No
        })
        .region(
            RegionId(1),
            part(|state| state)
                .role(Role::Button)
                .name("Region")
                .on_press(move || region.update(|n| *n += 1)),
        );
        let view = node()
            .role(Role::Button)
            .name("Parent")
            .on_press(move || parent.update(|n| *n += 1))
            .child(surface);
        ((claim, region, parent, seen), view)
    })
}

fn click(screen: &Screen, at: Point, batched: bool) {
    let rect = screen
        .get_by_role(Role::Canvas, "Surface")
        .rect()
        .expect("the canvas is laid out");
    let (x, y) = (rect.x + at.x, rect.y + at.y);
    if batched {
        // Down and Up reach the client in one turn.
        screen.click_at(x, y);
    } else {
        screen.mouse_move(x, y);
        screen.mouse_down(MouseButton::Primary);
        screen.mouse_up(MouseButton::Primary);
    }
}

#[test]
fn a_press_that_starts_a_region_gesture_does_not_activate_but_other_presses_do() {
    for batched in [false, true] {
        let (screen, (claim, region, parent, _)) = claiming_canvas();
        // The fallback claims the press of region 1: its click activates nothing.
        click(&screen, Point::new(20., 20.), batched);
        assert_eq!((region.get(), parent.get()), (0, 0), "batched: {batched}");
        // A press outside every region starts no gesture: the node's own action runs.
        click(&screen, Point::new(100., 60.), batched);
        assert_eq!((region.get(), parent.get()), (0, 1), "batched: {batched}");
        // The claim ended with its click: when the fallback lets the next
        // press of the same region go, the region activates.
        claim.set(false);
        screen.run_until_idle();
        click(&screen, Point::new(20., 20.), batched);
        assert_eq!((region.get(), parent.get()), (1, 1), "batched: {batched}");
        // Claiming again declines again, and only that click.
        claim.set(true);
        screen.run_until_idle();
        click(&screen, Point::new(20., 20.), batched);
        assert_eq!((region.get(), parent.get()), (1, 1), "batched: {batched}");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[test]
fn a_region_that_is_rebound_while_captured_gets_one_cancel() {
    let (screen, (visible, log)) = mount(|| {
        let visible = signal(true);
        let log = signal(Vec::<(u32, PointerPhase)>::new());
        let builds = Rc::new(Cell::new(0_u32));
        let root = canvas(move |draw, _| {
            if visible.get() {
                draw.region(RegionId(1), Rect::new(0., 0., 80., 40.));
            }
        })
        .name("Canvas")
        .role(Role::Canvas)
        .style(style().size(120, 80))
        .regions(
            move || {
                if visible.get() {
                    vec![1_u64]
                } else {
                    Vec::new()
                }
            },
            |id| RegionId(*id),
            move |_, _| {
                builds.set(builds.get() + 1);
                let binding = builds.get();
                part(|state| state)
                    .role(Role::Group)
                    .name("Region")
                    .on_pointer(move |event| {
                        log.update(|log| log.push((binding, event.phase)));
                        Handled::No
                    })
            },
        );
        ((visible, log), root)
    });
    screen.mouse_move(20., 20.);
    screen.mouse_down(MouseButton::Primary);
    visible.set(false);
    screen.run_until_idle();
    visible.set(true);
    screen.run_until_idle();
    screen.mouse_move(30., 25.);
    screen.mouse_up(MouseButton::Primary);
    let log = log.get();
    let of = |binding: u32, phase: PointerPhase| {
        log.iter()
            .filter(|entry| **entry == (binding, phase))
            .count()
    };
    assert_eq!(of(1, PointerPhase::Down), 1, "{log:?}");
    assert_eq!(of(1, PointerPhase::Cancel), 1, "{log:?}");
    assert_eq!(of(1, PointerPhase::Up), 0, "{log:?}");
    for phase in [
        PointerPhase::Down,
        PointerPhase::Move,
        PointerPhase::Up,
        PointerPhase::Cancel,
    ] {
        assert_eq!(
            of(2, phase),
            0,
            "the new binding never held the gesture: {log:?}"
        );
    }
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
