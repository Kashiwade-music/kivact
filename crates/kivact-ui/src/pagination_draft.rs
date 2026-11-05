//! Role: Own editable one-based page text and commit it atomically.
use kivact_core::state::{Binding, Machine, Step, Value, binding, machine, memo};
use kivact_core::view::{Handled, IntoView, Key, NodeBuilder, View, node_ref};
#[derive(Clone, PartialEq)]
enum PageDraft {
    Following(u32),
    Editing(String),
}
enum Event {
    Page,
    Typed(String),
    Commit,
    Focus(bool),
}
struct Snapshot {
    page: usize,
    count: usize,
}
impl Machine for Snapshot {
    type State = PageDraft;
    type Event = Event;
    type Effect = usize;
    fn transition(&self, state: &PageDraft, event: Event) -> Step<PageDraft, Event, usize> {
        let following =
            || PageDraft::Following(self.page.saturating_add(1).min(u32::MAX as usize) as u32);
        match event {
            Event::Typed(text) => Step::to(PageDraft::Editing(text)),
            Event::Commit | Event::Focus(false) => match state {
                PageDraft::Editing(text) => {
                    let page = text
                        .parse::<usize>()
                        .unwrap_or(self.page.saturating_add(1))
                        .saturating_sub(1)
                        .min(self.count.saturating_sub(1));
                    Step::to(PageDraft::Following(
                        page.saturating_add(1).min(u32::MAX as usize) as u32,
                    ))
                    .with(page)
                }
                PageDraft::Following(_) => Step::to(following()),
            },
            Event::Page => Step::to(following()),
            Event::Focus(true) => Step::to(state.clone()),
        }
    }
}
pub(super) fn input(
    page: Binding<usize>,
    count: Value<usize>,
    label: Value<String>,
    disabled: Value<bool>,
) -> View {
    let draft = machine(
        PageDraft::Following(
            page.get_untracked()
                .saturating_add(1)
                .min(u32::MAX as usize) as u32,
        ),
        move || Snapshot {
            page: page.get(),
            count: count.get(),
        },
        move |_, next| page.set(next),
    );
    draft.feed(memo(move || (page.get(), count.get())), |_| Event::Page);
    let reference = node_ref();
    draft.feed(reference.focused(), Event::Focus);
    let edit = binding(
        move || match draft.get() {
            PageDraft::Following(page) => page.to_string(),
            PageDraft::Editing(text) => text,
        },
        move |text| draft.send(Event::Typed(text)),
    );
    crate::text_input(label, edit)
        .node_ref(reference)
        .disabled(disabled)
        .on_key(move |event| {
            if event.key() == Key::Enter {
                draft.send(Event::Commit);
                Handled::Yes
            } else {
                Handled::No
            }
        })
        .into_view()
}
