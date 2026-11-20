//! Role: These tests verify the `#[component]` attribute: the props builder
//! it makes from one component function.
//!
//! Required props are the builder's positional arguments; ``
//! and `#[prop(default = ..)]` props are named setters; text borrows only for
//! the call, and a `ViewFn` prop is created again on every opening.
//! Unused variables are errors here: `old_card` proves that `#[component]`
//! keeps its `#[allow(..)]`.
#![deny(unused_variables)]

use kivact::Prop;
use kivact::prelude::*;
use kivact::test::render;
use kivact::view::KeyCode;
use kivact::view::Modifiers;
use kivact_public_api_tests::support::mount;
use std::sync::atomic::{AtomicU32, Ordering};

/// A card with a title and one content slot.
#[component]
fn card(title: Value<String>, children: View) -> impl Element {
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
    on_close: Option<Callback<()>>,
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
fn panel(title: Value<String>, open: Signal<bool>, content: kivact::view::ViewFn) -> impl IntoView {
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
fn picker(on_pick: Option<Callback<i32>>) -> impl IntoView {
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
    subtitle: Option<Value<String>>,
    level: Option<Value<u32>>,
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

#[component]
fn signed_picker(on_pick: Callback<i32>) -> impl IntoView {
    button("pick").on_press(move || on_pick.call(-3))
}

/// A closure passed to a `Callback<E>` prop gets its parameter type from
/// the prop, so it can call methods on it.
#[test]
fn callback_prop_closure_infers_its_parameter() {
    let (screen, seen) = mount(|| {
        let seen = signal(0);
        (seen, signed_picker(move |n| seen.set(n.abs())))
    });
    screen.get_by_role(Role::Button, "pick").click();
    assert_eq!(seen.get_untracked(), 3);
}

/// An element component: its root takes the caller's style and handlers,
/// and the caller's handler runs before the body's.
#[component]
fn tile(label: Value<String>, log: Signal<Vec<&'static str>>) -> impl Element {
    node()
        .role(Role::Button)
        .child(text(label))
        .style(style().width(10))
        .on_key(move |_| {
            log.update(|entries| entries.push("body"));
            Handled::No
        })
}

#[test]
fn an_element_component_takes_the_callers_style_and_handlers() {
    let (screen, log) = mount(|| {
        let log = signal(Vec::<&'static str>::new());
        let view = tile("A", log)
            .style(style().width(40))
            .test_id("tile")
            .focusable(true)
            .on_key(move |_| {
                log.update(|entries| entries.push("caller"));
                Handled::No
            });
        (log, view)
    });
    let width = screen
        .get_by_test_id("tile")
        .rect()
        .expect("laid out")
        .width;
    assert_eq!(width, 40.0);
    screen.get_by_test_id("tile").focus();
    screen.key_down(Key::Enter, KeyCode::Enter, Modifiers::default());
    assert_eq!(log.get_untracked(), vec!["caller", "body"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// An element component used where a labelled control is expected.
#[component]
fn my_input() -> impl Element {
    node()
        .role(Role::Button)
        .focusable(true)
        .test_id("my")
        .style(style().width(20))
}

#[test]
fn an_element_component_fits_in_field() {
    let screen = render(|| kivact::ui::field("Name", my_input()));
    screen.get_by_text("Name");
    screen.get_by_test_id("my");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A column of buttons, one per item; a press stores the item and tells the
/// caller through `on_pick`.
#[component]
fn choice<K: Clone + PartialEq + std::fmt::Display>(
    items: Value<Vec<K>>,
    value: Signal<Option<K>>,
    on_pick: Option<Callback<K>>,
) -> impl Element {
    let buttons: Vec<View> = items
        .get_untracked()
        .into_iter()
        .map(|item| {
            let label = item.to_string();
            button(label)
                .on_press(move || {
                    value.set(Some(item.clone()));
                    if let Some(on_pick) = on_pick {
                        on_pick.call(item.clone());
                    }
                })
                .into_view()
        })
        .collect();
    column(buttons)
}

#[test]
fn a_payload_callback_prop_infers_its_closure_parameter_in_a_generic_component() {
    let (screen, log) = mount(|| {
        let log = signal(Vec::<u32>::new());
        let view = choice(vec![1u32, 2, 3], signal(None::<u32>))
            .on_pick(move |k| log.update(|l| l.push(k)));
        (log, view)
    });
    screen.get_by_role(Role::Button, "2").click();
    assert_eq!(log.get_untracked(), vec![2]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_generic_component_takes_its_type_parameter_from_its_props() {
    let (screen, (picked, log)) = mount(|| {
        let picked = signal(None::<u32>);
        let log = signal(Vec::<u32>::new());
        let view =
            choice(vec![1u32, 2, 3], picked).on_pick(move |k: u32| log.update(|l| l.push(k)));
        ((picked, log), view)
    });
    screen.get_by_role(Role::Button, "2").click();
    assert_eq!(picked.get_untracked(), Some(2));
    assert_eq!(log.get_untracked(), vec![2]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A label with a number that the type parameter `T` supplies.
#[component]
fn zero_of<T: Default + std::fmt::Display + Clone + PartialEq>(
    label: Value<String>,
) -> impl Element {
    let v = signal(T::default());
    text(move || format!("{} {}", label.get(), v.get()))
}

#[test]
fn a_type_parameter_no_prop_names_is_kept_in_a_phantom_field() {
    let screen = render(|| zero_of::<u8>("n"));
    screen.get_by_text("n 0");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A label above a control, the control's type taken from the argument.
#[component]
fn labelled<C: Element>(label: Value<String>, control: C) -> impl Element {
    column((text(label), control))
}

#[test]
fn a_component_taking_its_type_parameter_as_a_prop_infers_it() {
    let screen = render(|| {
        column((
            labelled("Name", button("Go")),
            labelled("Box", node().test_id("n")),
        ))
    });
    screen.get_by_text("Name");
    screen.get_by_role(Role::Button, "Go");
    screen.get_by_text("Box");
    screen.get_by_test_id("n");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A focusable node whose key handler is the caller's handler.
#[component]
fn keys(on_key_prop: Callback<KeyEvent>) -> impl Element {
    node()
        .role(Role::Button)
        .focusable(true)
        .test_id("k")
        .on_key(move |event| on_key_prop.invoke(event))
}

/// A focusable node whose key handler is a shared `Callback<()>`.
#[component]
fn press_through(on_go: Callback<()>) -> impl Element {
    node()
        .role(Role::Button)
        .focusable(true)
        .test_id("p")
        .on_key(move |_| on_go.invoke(()))
}

#[test]
fn a_callback_prop_keeps_the_handled_result() {
    for (handled, parent_expected) in [(Handled::Yes, false), (Handled::No, true)] {
        let (screen, parent_saw) = mount(move || {
            let parent_saw = signal(false);
            let view = column((keys(move |_| handled),)).on_key(move |_| {
                parent_saw.set(true);
                Handled::No
            });
            (parent_saw, view)
        });
        screen.get_by_test_id("k").focus();
        screen.key_down(Key::Enter, KeyCode::Enter, Modifiers::default());
        assert_eq!(parent_saw.get_untracked(), parent_expected, "{handled:?}");
    }
    let (screen, parent_saw) = mount(|| {
        let parent_saw = signal(false);
        let go = callback(|_: ()| Handled::Yes);
        let view = column((press_through(go),)).on_key(move |_| {
            parent_saw.set(true);
            Handled::No
        });
        (parent_saw, view)
    });
    screen.get_by_test_id("p").focus();
    screen.key_down(Key::Enter, KeyCode::Enter, Modifiers::default());
    assert!(!parent_saw.get_untracked());
}

/// A plain value with a generic type; its `Value` comes from `#[derive(Prop)]`.
#[derive(Clone, Debug, PartialEq, Prop)]
struct Pair<T: Clone + PartialEq + std::fmt::Debug>
where
    T: Default,
{
    a: T,
}

#[component]
fn shows_pair(pair: Value<Pair<u8>>) -> impl Element {
    text(move || format!("{}", pair.get().a))
}

#[test]
fn derive_prop_on_a_generic_type() {
    let screen = render(|| shows_pair(Pair { a: 7 }));
    screen.get_by_text("7");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// A generic component whose bound sits in a where-clause.
#[component]
fn shown<T>(value: Value<T>) -> impl Element
where
    T: Clone + PartialEq + std::fmt::Display,
{
    text(move || value.get().to_string())
}

#[test]
fn a_generic_component_with_a_where_clause() {
    let (screen, n) = mount(|| {
        let n = signal(1_i32);
        let view = column((shown::<f64>(3.5), shown::<i32>(move || n.get())));
        (n, view)
    });
    screen.get_by_text("3.5");
    screen.get_by_text("1");
    n.set(4);
    screen.run_until_idle();
    screen.get_by_text("4");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
