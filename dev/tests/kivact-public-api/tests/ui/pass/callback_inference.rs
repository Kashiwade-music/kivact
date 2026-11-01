#![allow(dead_code, unused_variables)]
// `callback` takes one form, `Fn(T) -> R`: a callback without input is
// `|_| ..` (`T = ()`, called with `.call(())`), and a parameter that is only
// passed on or used where its type is fixed needs no annotation.
use kivact::Callback;
use kivact::prelude::*;
use kivact::view::callback;

fn labelled(on_pick: Callback<String>, on_reset: Callback<()>) -> impl IntoView {
    node().child(text("labelled"))
}

fn screen() -> impl IntoView {
    let log = signal(Vec::<String>::new());
    let count = signal(0u32);
    let view = labelled(
        callback(move |s| log.update(|l| l.push(s))),
        callback(move |_| count.set(0)),
    );
    let typed: Callback<u32> = callback(move |v| count.set(v));
    typed.call(2);
    let reset = callback(|_| ());
    reset.call(());
    view
}

fn main() {}
