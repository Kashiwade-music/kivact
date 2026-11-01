//! Role: One log source pane: search, level filter, timeline, lazy list and detail.
//!
//! One log source: search, level filter, timeline, lazy list (in the
//! generic `monospace` font family) and detail.

use kivact::style::current_theme;
use kivact::view::callback;
use std::rc::Rc;

use kivact::prelude::*;
use kivact_external_component::skin::{self, look};
use kivact_external_component::{Span, lazy_list, search_box, timeline};

use super::data::{BUCKET_MINUTES, Level, LogLine, MINUTES, clock};

/// The pane titled `title` over `lines`.
pub fn log_pane(title: &'static str, lines: Signal<Rc<Vec<LogLine>>>) -> impl IntoView {
    let look = look();
    let query = signal(String::new());
    let level = signal(Some(Level::Info));
    let range = signal(None::<Span>);
    let selected = signal(None::<u32>);
    let min_level = move || level.get().unwrap_or(Level::Info);
    // Lines matching the text and level filters (ids).
    let matching = memo(move || {
        let needle = query.with(|q| q.trim().to_lowercase());
        let min = min_level();
        lines.with(|all| {
            all.iter()
                .filter(|l| l.level >= min && l.matches(&needle))
                .map(|l| l.id)
                .collect::<Vec<u32>>()
        })
    });
    let buckets = (MINUTES / BUCKET_MINUTES) as usize;
    let histogram = memo(move || {
        let mut h = vec![0.0f32; buckets];
        lines.with(|all| {
            matching.with(|ids| {
                for id in ids {
                    let b = (all[*id as usize].minute / BUCKET_MINUTES) as usize;
                    h[b.min(buckets - 1)] += 1.0;
                }
            })
        });
        h
    });
    // Restricted to the selected time range.
    let visible = memo(move || {
        let span = range.get();
        matching.with(|ids| match span {
            None => ids.clone(),
            Some(s) => lines.with(|all| {
                ids.iter()
                    .copied()
                    .filter(|id| s.contains((all[*id as usize].minute / BUCKET_MINUTES) as usize))
                    .collect()
            }),
        })
    });
    let status = memo(move || {
        format!(
            "{} of {} lines",
            visible.with(|v| v.len()),
            lines.with(|l| l.len())
        )
    });
    let line = move |id: u32| lines.with(|all| all.get(id as usize).cloned());
    let detail = move || match selected.get().and_then(line) {
        Some(l) => format!(
            "#{} {} [{}] {}: {}",
            l.id,
            l.clock(),
            l.level.tag(),
            l.source,
            l.message
        ),
        None => "Select a line (click, ↑/↓, Enter)".to_string(),
    };
    let open = callback(move |id| selected.set(Some(id)));
    column((
        text(title).style(style().font_size(16).weight(600)),
        search_box("Search", query)
            .placeholder("text or source, e.g. timeout")
            .status(status)
            .test_id("search"),
        row((
            kivact::ui::select(
                "Level",
                vec![
                    ListItem::new(Level::Info, "All levels"),
                    ListItem::new(Level::Warn, "Warnings and errors"),
                    ListItem::new(Level::Error, "Errors only"),
                ],
                level,
            )
            .style(move || {
                let t = &current_theme();
                skin::field(look(), t)
            }),
            spacer(),
            show(
                move || range.get().is_some(),
                move || {
                    button("Clear range")
                        .variant(ButtonVariant::Secondary)
                        .on_press(move || range.set(None))
                },
            ),
        ))
        .style(style().gap(8).align_items(Align::Center)),
        timeline("Lines per 5 minutes", histogram, range)
            .bucket_label(|b| clock(b as u16 * BUCKET_MINUTES))
            .height(56.0)
            .test_id("timeline"),
        lazy_list(
            "Log lines",
            visible,
            |id: &u32| *id,
            move |_id: u32, id: Value<u32>| log_row(lines, id),
        )
        .row_height(24.0)
        .selection(selected)
        .on_activate(open)
        .item_name(move |id: &u32| format!("line {id}"))
        // Fixed viewport so that the visible rows (semantics) do not depend
        // on the skin's type size. The rows inherit the monospace family
        // (the OS's first installed candidate, e.g. Cascadia Mono on Windows).
        .style(style().height(360).shrink(0).font_family("monospace"))
        .test_id("lines"),
        text(detail)
            .role(Role::Status)
            .test_id("detail")
            .style(style().text_wrap(false)),
    ))
    .style(move || {
        let t = &current_theme();
        skin::panel(look(), t)
    })
    .style(style().grow(1).basis(0).min_width(0).min_height(0))
}

/// Content of one list row (its state belongs to the row).
fn log_row(lines: Signal<Rc<Vec<LogLine>>>, id: Value<u32>) -> impl IntoView {
    let look = look();
    let entry = memo(move || {
        let i = id.get() as usize;
        lines.with(|all| all.get(i).cloned())
    });
    let level = memo(move || entry.with(|l| l.as_ref().map(|l| l.level)));
    (
        text(move || entry.with(|l| l.as_ref().map(|l| l.clock()).unwrap_or_default()))
            .style(style().color(ColorToken::TextMuted).text_wrap(false)),
        text(move || level.get().map(|l| l.tag()).unwrap_or("").to_string())
            .style(move || {
                let t = &current_theme();
                let token = match level.get() {
                    Some(Level::Error) => ColorToken::Danger,
                    Some(Level::Warn) => ColorToken::Warning,
                    _ => ColorToken::Success,
                };
                skin::chip(look(), t, token)
            })
            .style(style().text_wrap(false)),
        text(move || {
            entry.with(|l| {
                l.as_ref()
                    .map(|l| format!("{}: {}", l.source, l.message))
                    .unwrap_or_default()
            })
        })
        .style(style().text_wrap(false)),
    )
}
