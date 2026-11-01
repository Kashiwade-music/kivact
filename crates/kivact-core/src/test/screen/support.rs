//! Role: Test helper utilities and fixtures.
//!
//! Shared support of the screen tests: the element they press, and the
//! deterministic random driver of `host_sync_tests` (the random probe tests
//! over the public API have their own in `tests/e2e/support/random.rs`).

use crate::Binding;
use crate::prelude::*;

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

// ---------------------------------------------------------------------------
// deterministic random driver

pub(super) struct Rng(u64);

impl Rng {
    pub(super) fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1)
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    pub(super) fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next() % n as u64) as usize
        }
    }
}

pub(super) fn seeds() -> std::ops::Range<u64> {
    0..40
}

pub(super) fn editor(label: impl IntoText, binding: impl Into<Binding<String>>) -> Node {
    use crate::primitive::{NodeRefExt, TextEditing, part};
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
