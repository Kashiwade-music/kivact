#![allow(dead_code, unused_variables)]
// Signatures: IntoValue (closures and concrete types coexist), HandlerResult `?`
// inference, UiError conversions, Copy of every public handle.
use kivact::prelude::*;
use kivact::{Binding, Callback};
use kivact::task::{Task};
use kivact::view::{Handled, KeyEvent, callback};
use kivact::view::PointerEvent;
use kivact::style::Color;
use kivact::draw::Draw;
use kivact::draw::Rect;
use kivact::view::PointerId;
use kivact::draw::Size;
include!("../support/prelude_stubs.rs");

fn assert_copy<T: Copy>() {}

fn into_values() -> impl IntoView {
    let s = signal(1.0f32);
    let v: Value<f32> = s.into_value();
    let label = signal(String::from("x"));
    column((
        text("static"),
        text(String::from("owned")),
        text(label),
        text(label.into_value()),
        text(move || label.get()),
        {
            let label = ("b").into_value();
            node()
                .role(Role::Button)
                .focusable(true)
                .name(label)
                .child(text(label))
        }
        .disabled(true)
        .disabled(move || s.get() > 1.0),
        node()
            .style(style().width(10).height(2.5).padding(1.0f32).gap(3u32))
            .style(move || style().opacity(v.get())),
        for_each(
            vec![1u32, 2, 3],
            |n| *n,
            |_, n| text(move || n.get().to_string()),
        ),
        show(move || true, || "yes"),
        show_some(move || Some(3u8), |n| text(move || n.get().to_string())),
    ))
}

fn handler_results() -> impl IntoView {
    let n = signal(0i32);
    let cb = callback(move |s: String| -> kivact::Result<()> {
        n.set(s.parse::<i32>()?);
        Ok(())
    });
    column((
        // `?` inside a handler infers `Result<(), UiError>` without annotations.
        {
            let label = ("parse").into_value();
            node()
                .role(Role::Button)
                .focusable(true)
                .name(label)
                .child(text(label))
        }
        .on_press(move || {
            let v: i32 = "42".parse()?;
            n.set(v);
            Ok(())
        }),
        {
            let label = ("plain").into_value();
            node()
                .role(Role::Button)
                .focusable(true)
                .name(label)
                .child(text(label))
        }
        .on_press(move || n.set(1)),
        node().on_key(move |k: KeyEvent| {
            let _v: i32 = "1".parse()?;
            Ok(Handled::Yes)
        }),
        node().on_key(move |_k| ()),
        kivact::control::node().name("t").part(kivact::interaction::use_text_edit(kivact::signal(String::new())).on_submit(move || cb.try_call("1".into())).control()),
    ))
}

fn errors() -> kivact::Result<()> {
    let e: UiError = std::io::Error::other("x").into();
    let m = UiError::msg("message");
    let b = UiError::from_boxed(Box::<dyn std::error::Error + Send + Sync>::from("boxed"));
    let _ = (e.to_string(), m.clone(), b.kind());
    std::fs::metadata("definitely-missing-file")?;
    Ok(())
}

fn copies() {
    assert_copy::<Signal<String>>();
    assert_copy::<Value<String>>();
    assert_copy::<Var<String>>();
    assert_copy::<Callback<String>>();
    assert_copy::<Binding<String>>();
    assert_copy::<kivact::Selector<u64>>();
    assert_copy::<NodeRef>();
    assert_copy::<Resource<String>>();
    assert_copy::<Action<String, String>>();
    assert_copy::<Action<String>>();
    assert_copy::<Task>();
    assert_copy::<kivact::motion::FrameLoop>();
    assert_copy::<kivact::motion::Animated<f32>>();
    assert_copy::<kivact::interaction::Scalar>();
    assert_copy::<kivact::interaction::SplitterApi>();
    assert_copy::<kivact::interaction::Drag<f64>>();
    assert_copy::<kivact::interaction::List<String>>();
    assert_copy::<kivact::interaction::ListSelection<String>>();
    assert_copy::<kivact::interaction::Tree<String>>();
    assert_copy::<kivact::interaction::Popup>();
    assert_copy::<kivact::interaction::PopupState>();
    assert_copy::<kivact::interaction::Toggle>();
    assert_copy::<kivact::interaction::Disclosure>();
    assert_copy::<kivact::interaction::DisclosureState>();
    assert_copy::<kivact::interaction::TooltipApi>();
    assert_copy::<kivact::interaction::DialogApi>();
}

fn interaction_signatures() -> impl IntoView {
    use kivact::ui::{ListItem, ScalarRange};
    use kivact::interaction::{DragCoordinates, ListPolicy, ListSelection, use_drag, use_list, use_scalar};
    let value = signal(0.0);
    let slider = use_scalar(value, ScalarRange::new(0.0, 1.0).unwrap())
        .key_step(0.05)
        .unwrap();
    let pointer = use_drag::<f64>(DragCoordinates::Local).disabled(signal(false));
    let selected = signal(None::<String>);
    let group = use_list(
        vec![ListItem::new("one".to_string(), "")],
        ListSelection::Single((selected).into()),
    )
    .policy(ListPolicy::radio())
    .policy(ListPolicy::tabs());
    column((
        node()
            .part(slider.thumb())
            .part(pointer.source(0.5))
            .name("Slider"),
        node()
            .part(slider.thumb())
            .role(Role::SpinButton)
            .name("Spinner"),
        node().part(group.list().role(Role::TabList)).child(
            node()
                .part(group.item("one".into()).role(Role::Tab).name("One"))
                .child(text("One")),
        ),
    ))
}

fn main() {
    let _ = into_values;
    let _ = handler_results;
    let _ = errors;
    let _ = interaction_signatures;
    copies();
}
