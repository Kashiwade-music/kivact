//! Role: Present pagination facts as the page text, the item range and the polite page status.
use kivact::state::Value;
use kivact::style::{Position, style};
use kivact::ui::theme::PaginationSummary;
use kivact::view::{LiveRegion, Node, Role, node, text};

fn page_messages(facts: Value<PaginationSummary>) -> Value<String> {
    let known = kivact::i18n::message_with(
        "pagination.page_of_total",
        "Page {current} of {total}",
        move || {
            [
                ("current", facts.get().page.saturating_add(1).to_string()),
                ("total", facts.get().count.to_string()),
            ]
        },
    );
    let current =
        kivact::i18n::message_with("pagination.page_announce", "Page {current}", move || {
            [("current", facts.get().page.saturating_add(1).to_string())]
        });
    kivact::state::memo(move || {
        if facts.get().count == 0 && facts.get().has_more {
            current.get()
        } else {
            known.get()
        }
    })
}
/// Keep the original polite page notice without another navigation or announcement owner.
pub fn pagination_status(facts: Value<PaginationSummary>) -> Node {
    node()
        .role(Role::Status)
        .live_region(LiveRegion::Polite)
        .style(
            style()
                .position(Position::Absolute)
                .size(1, 1)
                .opacity(0.0)
                .clip(true),
        )
        .child(text(page_messages(facts)))
}
