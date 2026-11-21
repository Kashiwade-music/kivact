//! Role: Own editable one-based page text and commit it atomically.
use kivact_core::state::{Signal, Value};
use kivact_core::view::{Element, Handled, IntoView, Key, View, node_ref};
use kivact_interaction::{DraftOptions, use_draft};

pub(super) fn input(
    page: Signal<usize>,
    count: Value<usize>,
    label: Value<String>,
    disabled: Value<bool>,
) -> View {
    let page_text = |page: &usize| page.saturating_add(1).min(u32::MAX as usize).to_string();
    let draft = use_draft(
        page,
        DraftOptions::new(page_text, move |text: &String| {
            let current = page.get_untracked();
            let typed = text.parse::<usize>().unwrap_or(current.saturating_add(1));
            Some(
                typed
                    .saturating_sub(1)
                    .min(count.get_untracked().saturating_sub(1)),
            )
        }),
    );
    let reference = node_ref();
    draft.settle_on_blur(reference.focused());
    crate::text_input(draft.editor())
        .label(label)
        .node_ref(reference)
        .disabled(disabled)
        .on_key(move |event| {
            if event.key() == Key::Enter {
                draft.commit();
                Handled::Yes
            } else {
                Handled::No
            }
        })
        .into_view()
}
