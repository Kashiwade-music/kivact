//! Role: These tests verify the `#[component]` attribute: the props builder
//! it makes from one component function.
//!
//! Required props are the builder's positional arguments; `#[prop(optional)]`
//! and `#[prop(default = ..)]` props are named setters; text borrows only for
//! the call, and a `ViewFn` prop is created again on every opening.

use crate::support::mount;
use kivact::prelude::*;
use kivact::test::render;

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
