//! Role: Present pagination facts as the page text, the item range and the polite page status.
use kivact::state::Value;
use kivact::style::{Position, style};
use kivact::ui::theme::PaginationSummary;
use kivact::view::{IntoView, LiveRegion, Node, Role, node, text};

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
/// Present a page number and known total from the canonical actual source.
pub fn pagination_page_summary(facts: Value<PaginationSummary>) -> impl IntoView {
    text(page_messages(facts))
}
/// Present the item range when configured, otherwise the actual page summary.
pub fn pagination_count(facts: Value<PaginationSummary>) -> impl IntoView {
    let page = page_messages(facts);
    let range = kivact::i18n::message_with(
        "pagination.item_count",
        "{from}–{to} of {total}",
        move || {
            let facts = facts.get();
            let total = facts.total_items.unwrap_or(0);
            let width = u64::from(facts.page_size);
            let from = (facts.page as u64)
                .saturating_mul(width)
                .saturating_add(1)
                .min(total);
            let to = (facts.page as u64)
                .saturating_add(1)
                .saturating_mul(width)
                .min(total);
            [
                ("from", from.to_string()),
                ("to", to.to_string()),
                ("total", total.to_string()),
            ]
        },
    );
    text(move || {
        if facts.get().total_items.is_some() {
            range.get()
        } else {
            page.get()
        }
    })
}
/// Keep the original polite page notice without another navigation or announcement owner.
pub fn pagination_status(facts: Value<PaginationSummary>) -> Node {
    node()
        .role(Role::Status)
        .live_region(LiveRegion::Polite)
        .test_id("kivact.sr-only")
        .style(
            style()
                .position(Position::Absolute)
                .size(1, 1)
                .opacity(0.0)
                .clip(true),
        )
        .child(text(page_messages(facts)))
}
