//! Role: These tests verify the `#[component]` attribute: the props builder
//! it makes from one component function.
//!
//! Required props are the builder's positional arguments; `#[prop(optional)]`
//! and `#[prop(default = ..)]` props are named setters; text borrows only for
//! the call, and a `ViewFn` prop is created again on every opening.
//! Unused variables are errors here: `old_card` proves that `#[component]`
//! keeps its `#[allow(..)]`.
#![deny(unused_variables)]

use crate::support::mount;
use kivact::prelude::*;
use kivact::test::render;
use std::sync::atomic::{AtomicU32, Ordering};

/// A card with a title and one content slot.
#[component]
fn card(title: Value<String>, children: View) -> impl IntoView {
    column((text(title), children))
}

#[test]
fn required_props_are_positional() {
    let screen = render(|| card("Title", text("Body")));
    assert_eq!(screen.get_by_text("Title").text(), "Title");
    assert_eq!(screen.get_by_text("Body").text(), "Body");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A row with a default count and an optional close button.
#[component]
fn closable(
    #[prop(default = 3)] count: Value<i32>,
    #[prop(optional)] on_close: Option<Callback<()>>,
) -> impl IntoView {
    let close = on_close.map(|on_close| button("Close").on_press(on_close));
    column((text(move || format!("n={}", count.get())), close))
}

#[test]
fn optional_and_default_props_use_setters() {
    let (screen, ()) = mount(|| {
        let closed = signal(false);
        let view = column((
            closable(),
            closable().count(5),
            closable().count(7).on_close(move || closed.set(true)),
            text(move || format!("closed={}", closed.get())),
        ));
        ((), view)
    });
    screen.get_by_text("n=3");
    screen.get_by_text("n=5");
    screen.get_by_text("n=7");
    screen.get_by_text("closed=false");
    screen.get_by_role(Role::Button, "Close").click();
    screen.get_by_text("closed=true");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A panel whose body is created again each time it opens.
#[component]
fn panel(
    title: Value<String>,
    open: Binding<bool>,
    content: kivact::view::ViewFn,
) -> impl IntoView {
    column((text(title), show(open, content)))
}

#[test]
fn borrowed_text_and_recreated_content() {
    let (screen, open) = mount(|| {
        let open = signal(true);
        let name = String::from("Panel");
        let view = column((panel(name.as_str(), open, || text("Body")), text("outside")));
        (open, view)
    });
    screen.get_by_text("Panel");
    screen.get_by_text("Body");
    open.set(false);
    screen.run_until_idle();
    assert!(screen.query_by_text("Body").is_none());
    open.set(true);
    screen.run_until_idle();
    assert!(screen.query_by_text("Body").is_some());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn component_owner_is_disposed_on_unmount() {
    let (screen, visible) = mount(|| {
        let visible = signal(false);
        let view = column((show(visible, || card("T", text("B"))), text("outside")));
        (visible, view)
    });
    let before = screen.stats().live_owners;
    visible.set(true);
    screen.run_until_idle();
    assert!(screen.get_by_text("B").exists());
    assert!(screen.stats().live_owners > before);
    visible.set(false);
    screen.run_until_idle();
    assert_eq!(screen.stats().live_owners, before);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A button reporting a payload to a caller that wrote no type.
#[component]
fn picker(#[prop(optional)] on_pick: Option<Callback<i32>>) -> impl IntoView {
    button("Pick").on_press(move || {
        if let Some(cb) = on_pick {
            cb.call(7);
        }
    })
}

#[test]
fn payload_callback_infers_the_argument() {
    let (screen, ()) = mount(|| {
        let picked = signal(0);
        let view = column((
            picker().on_pick(move |n| picked.set(n)),
            text(move || format!("picked={}", picked.get())),
        ));
        ((), view)
    });
    screen.get_by_text("picked=0");
    screen.get_by_role(Role::Button, "Pick").click();
    screen.get_by_text("picked=7");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A heading with an optional subtitle and an optional level.
#[component]
fn heading(
    title: Value<String>,
    #[prop(optional)] subtitle: Option<Value<String>>,
    #[prop(optional)] level: Option<Value<u32>>,
) -> impl IntoView {
    let level = level.map(|level| text(move || format!("level={}", level.get())));
    column((text(title), subtitle.map(text), level))
}

#[test]
fn optional_value_props_are_none_or_reactive() {
    let (screen, subtitle) = mount(|| {
        let subtitle = signal(String::from("Second"));
        let view = column((
            heading("Plain"),
            heading("Full").subtitle(subtitle).level(2_u32),
        ));
        (subtitle, view)
    });
    screen.get_by_text("Plain");
    screen.get_by_text("Full");
    screen.get_by_text("Second");
    screen.get_by_text("level=2");
    assert_eq!(screen.all_by_text("level=2").len(), 1);
    subtitle.set(String::from("Changed"));
    screen.run_until_idle();
    screen.get_by_text("Changed");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// How many times [`counted_dismiss`] ran: the default expression of
/// `notice`'s `on_dismiss`.
static DISMISS_DEFAULTS: AtomicU32 = AtomicU32::new(0);

/// The default handler of `notice`, counting each run of the expression.
fn counted_dismiss() -> impl Fn() + 'static {
    DISMISS_DEFAULTS.fetch_add(1, Ordering::SeqCst);
    || {}
}

/// A notice with a default dismiss handler.
#[component]
fn notice(
    message: Value<String>,
    #[prop(default = counted_dismiss())] on_dismiss: Callback<()>,
) -> impl IntoView {
    column((text(message), button("Dismiss").on_press(on_dismiss)))
}

#[test]
fn callback_default_runs_when_the_builder_is_called() {
    let (screen, made_by_call) = mount(|| {
        let before = DISMISS_DEFAULTS.load(Ordering::SeqCst);
        let view = notice("Saved");
        (DISMISS_DEFAULTS.load(Ordering::SeqCst) - before, view)
    });
    assert_eq!(made_by_call, 1, "the default runs in the builder call");
    assert_eq!(
        DISMISS_DEFAULTS.load(Ordering::SeqCst),
        1,
        "mounting does not run the default again"
    );
    screen.get_by_text("Saved");
    screen.get_by_role(Role::Button, "Dismiss").click();
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A card kept for old callers, with the attributes `#[component]` keeps:
/// `allow` on every generated item, `deprecated` on the builder. The file's
/// `deny(unused_variables)` makes a dropped `allow` a compile error.
#[component]
#[allow(
    unused_variables,
    reason = "the unused name proves the allow reaches the body"
)]
#[deprecated(note = "use `card`")]
fn old_card(title: Value<String>) -> impl IntoView {
    let unused = 0;
    text(title)
}

#[test]
#[deny(unfulfilled_lint_expectations)]
#[expect(deprecated, reason = "#[component] keeps #[deprecated] on the builder")]
fn allow_and_deprecated_reach_the_generated_items() {
    let screen = render(|| old_card("Old"));
    screen.get_by_text("Old");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A count whose argument the body changes.
#[component]
fn bumped(mut start: i32) -> impl IntoView {
    start += 1;
    text(format!("start={start}"))
}

#[test]
fn mut_arguments_stay_mutable_in_the_body() {
    let screen = render(|| bumped(1));
    screen.get_by_text("start=2");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A button whose default handler writes no type.
#[component]
fn defaulted(#[prop(default = |n| { let _ = n.abs(); })] on_pick: Callback<i32>) -> impl IntoView {
    button("Pick").on_press(move || on_pick.call(-7))
}

#[test]
fn default_payload_callback_infers_the_argument() {
    let screen = render(defaulted);
    screen.get_by_role(Role::Button, "Pick").click();
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
