#![allow(dead_code, unused_variables)]
// Usage example: counter
use kivact::prelude::*;
use kivact::view::PointerEvent;
use kivact::style::Color;
use kivact::draw::Draw;
use kivact::draw::Rect;
use kivact::view::Handled;
use kivact::Callback;
use kivact::view::PointerId;
use kivact::draw::Size;
include!("../support/prelude_stubs.rs");

fn counter(initial: i32) -> impl IntoView {
    let count = signal(initial);
    let doubled = memo(move || count.get() * 2); // Value<i32>。等価なら下流を止める

    column((
        text(move || format!("count: {} / doubled: {}", count.get(), doubled.get())),
        { let label = ("増やす").into_value(); node().role(Role::Button).focusable(true).name(label).child(text(label)) }.on_press(move || count.update(|c| *c += 1)),
    ))
    .style(style().gap(8).padding(12))
}

fn app_main() -> kivact::Result<()> {
    App::new("Counter").run(|| counter(0))
}

fn main() {
    let _ = app_main;
}
