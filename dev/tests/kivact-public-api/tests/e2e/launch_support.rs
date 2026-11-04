//! Role: Test helper utilities and fixtures.
//!
//! Shared support of the screen tests: the element they press, and the
//! deterministic random driver of `host_sync_tests` (the random probe tests
//! over the public API have their own in `tests/e2e/support/random.rs`).

use kivact::Binding;
use kivact::prelude::{IntoText, IntoValue, Node, NodeBuilder, Role, node, text};

/// What the screen tests press: an element that is a button to assistive
/// technology (role Button, focusable, `label` as its name and text),
/// pressed through `A11yAction::Activate` like the standard button.
/// Component kits cannot be used here: they link their own `kivact`,
/// not the one under test.
#[track_caller]
pub(super) fn button_box(label: &'static str) -> Node {
    node()
        .role(Role::Button)
        .focusable(true)
        .name(label)
        .child(text(label))
}

pub(super) fn editor(label: impl IntoText, binding: impl Into<Binding<String>>) -> Node {
    use kivact::primitive::{NodeRefExt, TextEditing, part};
    let binding = binding.into();
    let value = binding.into_value();
    let control = part(|state| state).role(Role::TextBox).focusable(true);
    let reference = control.node_ref();
    let text = move || std::rc::Rc::<str>::from(value.get());
    node().name(label).part(
        control.text_editing(
            TextEditing::new(text)
                .report_text(true)
                .on_edit(move |edit| {
                    if edit.composition().is_some() {
                        return;
                    }
                    let Some(edited) = edit.text else {
                        return;
                    };
                    binding.set(edited.clone());
                    if let Some(value) = value.try_get()
                        && value != edited
                    {
                        reference.set_edit_value(value);
                    }
                }),
        ),
    )
}
