// A closure returning `&str` is text: it compiles for `text`, `.name(..)` and a
// component's `impl IntoText` parameter, next to a closure returning `String`.
use kivact::prelude::*;

fn label(on: Signal<bool>) -> Node {
    text(move || if on.get() { "on" } else { "off" })
}

fn named(on: Signal<bool>) -> Node {
    node()
        .name(move || if on.get() { "enabled" } else { "disabled" })
        .child(text(move || format!("{}", on.get())))
}

fn badge(caption: impl IntoText) -> Node {
    let caption = caption.into_text_value();
    node().name(caption).child(text(caption))
}

fn main() {
    let _ = (label, named);
    let _ = |on: Signal<bool>| badge(move || if on.get() { "on" } else { "off" });
}
