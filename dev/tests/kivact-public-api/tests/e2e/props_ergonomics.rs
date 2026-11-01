//! Role: Tests for property input and resource observation ergonomics.
//!
//! Owned property inputs and resource observations through the public
//! facade, including owner disposal.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use crate::support::mount;
use kivact::prelude::*;
use kivact::view::{Reactive, constant};

#[derive(Clone, PartialEq)]
struct CardProps {
    title: String,
}

kivact::view::constant_value!(CardProps);

fn card(props: impl IntoValue<CardProps>) -> impl IntoView {
    let props = props.into_value();
    let title = memo(move || props.with(|p| p.title.clone()));
    node().role(Role::Group).name(title).child(text(title))
}

/// A type its crate did not register: a caller passes it wrapped by `constant`.
#[derive(Clone, Copy, PartialEq)]
enum Lane {
    Done,
}

fn lane_badge(lane: impl IntoValue<Lane>) -> impl IntoView {
    let lane = lane.into_value();
    text(move || match lane.get() {
        Lane::Done => "lane done".to_owned(),
    })
}

fn small_numbers(
    byte: impl IntoValue<u8>,
    word: impl IntoValue<u16>,
    signed_byte: impl IntoValue<i8>,
    signed_word: impl IntoValue<i16>,
    letter: impl IntoValue<char>,
) -> impl IntoView {
    let (byte, word) = (byte.into_value(), word.into_value());
    let (signed_byte, signed_word) = (signed_byte.into_value(), signed_word.into_value());
    let letter = letter.into_value();
    text(move || {
        format!(
            "small {} {} {} {} {}",
            byte.get(),
            word.get(),
            signed_byte.get(),
            signed_word.get(),
            letter.get()
        )
    })
}

#[test]
fn constant_wraps_unregistered_types_and_small_numbers_and_chars_are_constants() {
    let (screen, ()) = mount(|| {
        (
            (),
            column((
                lane_badge(constant(Lane::Done)),
                small_numbers(1u8, 2u16, -3i8, -4i16, 'x'),
            )),
        )
    });
    screen.get_by_text("lane done");
    screen.get_by_text("small 1 2 -3 -4 x");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn borrowed_action(label: &str) -> Node {
    node().role(Role::Button).name(label).child(text(label))
}

#[test]
fn static_custom_props_and_borrowed_labels_coexist_with_reactive_inputs() {
    let (screen, (props, revision)) = mount(|| {
        let props = signal(CardProps {
            title: "old title".to_owned(),
        });
        let revision = signal(0);
        let label = String::from("borrowed label");
        let action = borrowed_action(&label);
        drop(label);
        (
            (props, revision),
            column((
                card(
                    (CardProps {
                        title: "constant card".to_owned(),
                    })
                    .into_value(),
                ),
                card(props),
                card(move || CardProps {
                    title: format!("revision {}", revision.get()),
                }),
                action,
            )),
        )
    });
    screen.get_by_role(Role::Group, "constant card");
    screen.get_by_role(Role::Group, "old title");
    screen.get_by_role(Role::Button, "borrowed label");
    screen.get_by_text("revision 0");
    props.set(CardProps {
        title: "new title".to_owned(),
    });
    revision.set(1);
    screen.run_until_idle();
    screen.get_by_role(Role::Group, "new title");
    screen.get_by_text("new title");
    screen.get_by_text("revision 1");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn badge(label: Reactive<String>) -> impl IntoView {
    let label = label.into_value();
    node().role(Role::Group).name(label).child(text(label))
}

#[test]
fn reactive_built_before_mount_becomes_a_value_in_setup() {
    // No App exists yet: building, cloning, reading and printing need no owner.
    let fixed = Reactive::new("fixed label");
    assert_eq!(fixed.get(), "fixed label");
    assert_eq!(fixed.get_untracked(), "fixed label");
    assert_eq!(format!("{:?}", fixed.clone()), "Reactive(\"fixed label\")");
    let computed = Reactive::new(|| "computed label".to_owned());
    assert_eq!(computed.get(), "computed label");
    assert_eq!(format!("{computed:?}"), "<reactive>");

    let (screen, name) = mount(move || {
        let name = signal("first".to_owned());
        let followed = Reactive::new(name);
        (
            name,
            column((badge(fixed), badge(computed), badge(followed))),
        )
    });
    screen.get_by_role(Role::Group, "fixed label");
    screen.get_by_role(Role::Group, "computed label");
    screen.get_by_role(Role::Group, "first");
    name.set("second".to_owned());
    screen.run_until_idle();
    screen.get_by_role(Role::Group, "second");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn caption_badge(caption: impl IntoText) -> impl IntoView {
    let caption = caption.into_text_value();
    node().role(Role::Group).name(caption).child(text(caption))
}

#[test]
fn text_inputs_take_every_text_form_including_closures_returning_str() {
    let (screen, (on, name)) = mount(|| {
        let on = signal(false);
        let name = signal("from signal".to_owned());
        let stored = Reactive::new("stored label");
        (
            (on, name),
            column((
                text(move || if on.get() { "on" } else { "off" }),
                caption_badge("constant"),
                caption_badge(String::from("owned")),
                caption_badge(std::rc::Rc::<str>::from("shared")),
                caption_badge(name),
                caption_badge(stored),
                caption_badge(move || {
                    if on.get() {
                        "closure on"
                    } else {
                        "closure off"
                    }
                }),
                caption_badge(move || format!("formatted {}", on.get())),
            )),
        )
    });
    for expected in [
        "off",
        "constant",
        "owned",
        "shared",
        "from signal",
        "stored label",
    ] {
        screen.get_by_text(expected);
    }
    screen.get_by_role(Role::Group, "closure off");
    screen.get_by_role(Role::Group, "formatted false");
    on.set(true);
    name.set("changed".to_owned());
    screen.run_until_idle();
    screen.get_by_text("on");
    screen.get_by_role(Role::Group, "closure on");
    screen.get_by_role(Role::Group, "formatted true");
    screen.get_by_role(Role::Group, "changed");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn resource_state_and_latest_track_requests() {
    let (screen, query) = mount(move || {
        let query = signal(1_u32);
        let rows = resource(
            move || query.get(),
            move |n| async move {
                kivact::task::sleep(Duration::from_millis(10)).await;
                if n == 3 {
                    Err(UiError::msg("request failed"))
                } else {
                    Ok((0..n).map(|i| format!("row {i}")).collect::<Vec<_>>())
                }
            },
        );
        (
            query,
            column((
                text(move || match rows.state() {
                    AsyncState::Ready(rows) => format!("ready {}", rows.len()),
                    AsyncState::Failed(_) => "failed".to_owned(),
                    _ => "loading".to_owned(),
                }),
                text(move || format!("latest {}", rows.latest().map_or(0, |r| r.len()))),
            )),
        )
    });
    screen.get_by_text("loading");
    screen.get_by_text("latest 0");
    screen.advance(Duration::from_millis(10));
    screen.get_by_text("ready 1");
    screen.get_by_text("latest 1");
    query.set(2);
    screen.run_until_idle();
    screen.get_by_text("loading");
    screen.get_by_text("latest 1");
    screen.advance(Duration::from_millis(10));
    screen.get_by_text("ready 2");
    screen.get_by_text("latest 2");
    query.set(3);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(10));
    screen.get_by_text("failed");
    screen.get_by_text("latest 2");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn resource_handles_do_not_extend_the_branch_owner_lifetime() {
    let leaked = Rc::new(Cell::new(None));
    let slot = leaked.clone();
    let cleanups = Rc::new(Cell::new(0));
    let count = cleanups.clone();
    let (screen, open) = mount(move || {
        let open = signal(false);
        (
            open,
            show(open, move || {
                let cleanup = count.clone();
                on_cleanup(move || cleanup.set(cleanup.get() + 1));
                let resource = resource(
                    || (),
                    |()| async {
                        kivact::task::sleep(Duration::from_millis(50)).await;
                        Ok(9_u32)
                    },
                );
                slot.set(Some(resource));
                text(move || match resource.state() {
                    AsyncState::Ready(n) => n.to_string(),
                    _ => "pending branch".to_owned(),
                })
            }),
        )
    });
    let baseline = screen.stats();
    open.set(true);
    screen.run_until_idle();
    screen.get_by_text("pending branch");
    let resource = leaked.get().expect("branch resource");
    open.set(false);
    screen.run_until_idle();
    assert_eq!(cleanups.get(), 1);
    assert_eq!(screen.stats().live_objects, baseline.live_objects);
    assert_eq!(screen.stats().timers, baseline.timers);
    assert!(std::panic::catch_unwind(|| resource.state()).is_err());
    assert!(std::panic::catch_unwind(|| resource.latest()).is_err());
}
