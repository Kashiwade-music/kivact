#![allow(dead_code, reason = "compile-only public component examples")]
// Every standard component in the prelude can be called with its required
// arguments using `use kivact::prelude::*;` alone.

use kivact::prelude::*;
use kivact::view::{Placement, overlay};

fn form() -> impl IntoView {
    let name = signal(String::new());
    let notes = signal(String::new());
    let agreed = signal(false);
    let dark = signal(false);
    let size = signal(Some("m"));
    let tab = signal("general");
    let open = signal(false);
    let sizes = vec![ListItem::new("s", "Small"), ListItem::new("m", "Medium")];
    let pages = vec![
        ListItem::new("general", "General"),
        ListItem::new("advanced", "Advanced"),
    ];
    let actions = vec![MenuEntry::item("copy", "Copy"), MenuEntry::item("paste", "Paste")];
    column((
        text_input("Name", name),
        text_area("Notes", notes),
        checkbox("I agree", agreed),
        switch("Dark mode", dark),
        select("Size", sizes, size),
        tabs("Tabs", pages, tab).content(|key| text(key)),
        menu("Edit", actions, |key| println!("{key}")),
        tooltip("Saves the form").attach(
            button("Save")
                .variant(ButtonVariant::Primary)
                .on_press(move || open.set(true)),
        ),
        dialog("Saved", open)
            .content(|| text("The form was saved."))
            .footer(move || button("Close").on_press(move || open.set(false))),
    ))
}

// The modifiers are the methods of `NodeBuilder`, in the prelude: a `Node`,
// an overlay and a virtual list take them with no further import. `role` and
// the states are `Node`'s own methods, not `NodeBuilder`'s.
fn modifiers() -> impl IntoView {
    let anchor = node_ref();
    let rows = signal(vec![1u32, 2, 3]);
    (
        node()
            .role(Role::Group)
            .style(style().padding(4))
            .name("Panel")
            .on_press(|| {}),
        overlay(anchor, Placement::Below, text("Menu")).name("Actions"),
        virtual_list(rows, |row| *row, |_, row| text(move || row.get().to_string()))
            .name("Rows")
            .style(style().grow(1)),
    )
}

fn main() {}
