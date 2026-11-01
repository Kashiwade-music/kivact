#![allow(dead_code, reason = "compile-only public component examples")]
// A closure is accepted wherever a component takes a callback, with no
// `callback` and no annotation on the closure parameter.

use kivact::ui::{EditPhase, MenuEntry, ScalarRange};
use kivact::interaction::{use_menu, use_splitter};
use kivact::prelude::*;

fn actions(last: Signal<Option<&'static str>>) -> impl IntoView {
    let entries = vec![
        MenuEntry::item("save", "Save"),
        MenuEntry::item("open", "Open"),
    ];
    let file = use_menu(entries, move |key| last.set(Some(key)));
    node().part(file.trigger()).name("File").child(text("File"))
}

fn split(edits: Signal<u32>) -> impl IntoView {
    let size = signal(240.0);
    let range = ScalarRange::new(120.0, 480.0).unwrap();
    node().name("Sidebar width").part(
        use_splitter(size, range, Orientation::Horizontal)
            .on_edit(move |phase| {
                if matches!(phase, EditPhase::Commit) {
                    edits.update(|n| *n += 1);
                }
            })
            .handle(),
    )
}

fn main() {}
