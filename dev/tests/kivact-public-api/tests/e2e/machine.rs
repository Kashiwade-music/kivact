//! Role: Exercise owner-scoped machines through the public facade.
use crate::support::mount;
use kivact::prelude::*;
use kivact::state::{Machine, Step, machine};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

struct Counter;
enum Event {
    Add,
    Arm(u32),
    Tick(u32),
    Input(u32),
    Chain,
}
impl Machine for Counter {
    type State = u32;
    type Event = Event;
    type Effect = bool;
    fn transition(&self, state: &u32, event: Event) -> Step<u32, Event, bool> {
        match event {
            Event::Add => Step::to(state + 1).with(false),
            Event::Arm(n) => Step::stay().after(Duration::from_millis(100), Event::Tick(n)),
            Event::Tick(n) => Step::to(state + n).with(false),
            Event::Input(n) => Step::to(state + n).with(false),
            Event::Chain => Step::to(state + 1).with(true).with(false),
        }
    }
}

#[test]
fn a_machine_renders_its_state_and_runs_effects_after_the_transition() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let out = log.clone();
    let (screen, counter) = mount(move || {
        let handle = Rc::new(RefCell::new(None::<kivact::state::MachineHandle<Counter>>));
        let observed = handle.clone();
        let counter = machine(
            0,
            || Counter,
            move |_, _| {
                out.borrow_mut().push(observed.borrow().unwrap().get());
            },
        );
        *handle.borrow_mut() = Some(counter);
        (counter, text(move || counter.get().to_string()))
    });
    counter.send(Event::Add);
    assert_eq!(&*log.borrow(), &[1]);
    assert_eq!(counter.get(), 1);
    screen.run_until_idle();
    screen.get_by_text("1");
}

#[test]
fn a_machine_timer_fires_once_and_is_replaced_by_the_same_event_variant() {
    let (screen, counter) = mount(|| {
        let counter = machine(0, || Counter, |_, _| {});
        (counter, text(move || counter.get().to_string()))
    });
    counter.send(Event::Arm(1));
    screen.advance(Duration::from_millis(60));
    counter.send(Event::Arm(7));
    screen.advance(Duration::from_millis(60));
    assert_eq!(counter.get(), 0);
    screen.advance(Duration::from_millis(60));
    assert_eq!(counter.get(), 7);
    screen.advance(Duration::from_secs(1));
    assert_eq!(counter.get(), 7);
    screen.get_by_text("7");
}

#[test]
fn feed_turns_an_input_change_into_one_event() {
    let (screen, (source, counter)) = mount(|| {
        let source = signal(2);
        let counter = machine(0, || Counter, |_, _| {});
        counter.feed(source, Event::Input);
        ((source, counter), text(move || counter.get().to_string()))
    });
    assert_eq!(counter.get(), 0);
    source.set(3);
    screen.run_until_idle();
    assert_eq!(counter.get(), 3);
    source.set(3);
    screen.run_until_idle();
    assert_eq!(counter.get(), 3);
}

#[test]
fn a_machine_timer_is_canceled_with_its_owner() {
    let calls = Rc::new(RefCell::new(0));
    let out = calls.clone();
    let (screen, open) = mount(move || {
        let open = signal(true);
        (
            open,
            show(open, move || {
                let out = out.clone();
                let counter = machine(0, || Counter, move |_, _| *out.borrow_mut() += 1);
                button("Arm").on_press(move || counter.send(Event::Arm(1)))
            }),
        )
    });
    screen.get_by_role(Role::Button, "Arm").click();
    open.set(false);
    screen.run_until_idle();
    screen.advance(Duration::from_secs(1));
    assert_eq!(*calls.borrow(), 0);
    assert!(screen.errors().is_empty());
}

#[test]
fn send_inside_a_transition_effect_is_queued_not_reentrant() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let out = log.clone();
    let (_screen, counter) = mount(move || {
        let counter = machine(
            0,
            || Counter,
            move |cx, chain| {
                if chain {
                    out.borrow_mut().push("before");
                    cx.send(Event::Add);
                    out.borrow_mut().push("after");
                } else {
                    out.borrow_mut().push("effect");
                }
            },
        );
        (counter, text(move || counter.get().to_string()))
    });
    counter.send(Event::Chain);
    assert_eq!(&*log.borrow(), &["before", "after", "effect", "effect"]);
    assert_eq!(counter.get(), 2);
}
