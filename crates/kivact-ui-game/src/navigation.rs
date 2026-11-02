//! Role: Numbered steps, breadcrumbs and pagination.
use kivact::i18n::message;
use kivact::style::current_theme;

use kivact::control::{Node, NodeBuilder, row, text};
use kivact::style::{Align, ColorToken, pct, style};
use kivact::view::{IntoValue, IntoView, Role, for_index, node_ref};
use kivact::{Binding, Value, var};

/// Connected numbered steps; indices before the current step are complete.
#[track_caller]
pub fn stepper(labels: impl IntoValue<Vec<String>>, current: impl IntoValue<usize>) -> Node {
    let labels = labels.into_value();
    let current = current.into_value();
    row(for_index(labels, move |i, label: Value<String>| {
        row((
            text(move || {
                if i < current.get() {
                    "✓".to_owned()
                } else {
                    format!("{:02}", i + 1)
                }
            })
            .style(move || {
                let t = &current_theme();
                style()
                    .font_family(t.get::<crate::tokens::Tokens>().display_family.as_str())
                    .font_size(18)
                    .weight(800)
                    .italic(true)
                    .color(if i == current.get() {
                        t.colors().accent_text
                    } else if i < current.get() {
                        t.colors().accent
                    } else {
                        t.get::<crate::tokens::Tokens>().text_3
                    })
            }),
            text(label).style(move || {
                let t = &current_theme();
                style()
                    .font_size(12)
                    .weight(700)
                    .color(if i == current.get() {
                        t.colors().accent_text
                    } else if i < current.get() {
                        t.colors().accent
                    } else {
                        t.get::<crate::tokens::Tokens>().text_3
                    })
            }),
        ))
        .style(move || {
            let t = &current_theme();
            let p = t.get::<crate::tokens::Tokens>();
            style()
                .height(40)
                .padding_left(if i == 0 { 14 } else { 22 })
                .padding_right(18)
                .gap(8)
                .align_items(Align::Center)
                .grow(1)
                .clip_path(
                    kivact::draw::Shape::polygon()
                        .point(0, 0)
                        .point(pct(100) - 12, 0)
                        .point(pct(100), pct(50))
                        .point(pct(100) - 12, pct(100))
                        .point(0, pct(100))
                        .point(if i == 0 { 0 } else { 12 }, pct(50)),
                )
                .background(if i == current.get() {
                    t.colors().accent
                } else if i < current.get() {
                    p.primary_weak
                } else {
                    p.surface_3
                })
                .color(if i == current.get() {
                    t.colors().accent_text
                } else if i < current.get() {
                    t.colors().accent
                } else {
                    p.text_3
                })
        })
    }))
    .style(style().width(pct(100)).gap(3))
}

/// Breadcrumbs separated by blue slash marks.
#[track_caller]
pub fn breadcrumb(items: impl IntoValue<Vec<String>>) -> Node {
    let items = items.into_value();
    row(kivact::view::dynamic(items, |items| {
        let n = items.len();
        items
            .into_iter()
            .enumerate()
            .map(|(i, label)| {
                row((
                    text(label).style(
                        style()
                            .font_size(12)
                            .weight(if i + 1 == n { 700 } else { 400 })
                            .color(if i + 1 == n {
                                ColorToken::Text
                            } else {
                                ColorToken::TextMuted
                            }),
                    ),
                    text(if i + 1 == n { "" } else { "/" })
                        .style(style().font_size(20).weight(700).color(ColorToken::Accent)),
                ))
                .style(style().gap(8).align_items(Align::Center))
                .into_view()
            })
            .collect::<Vec<_>>()
    }))
    .role(Role::Group)
    .style(style().gap(8).align_items(Align::Center))
}

/// Pagination including endpoints and a neighborhood with gap markers.
#[track_caller]
pub fn pagination(page: impl Into<Binding<usize>>, count: impl IntoValue<usize>) -> Node {
    let total = count.into_value();
    let binding = page.into();
    let page = binding.into_value();
    // Armed only when a press really changed the page, so the rebuild that
    // consumes it is certain.
    let restore_focus = var(false);
    let (previous_label, next_label) = (
        message("game.pagination.previous", "Previous"),
        message("game.pagination.next", "Next"),
    );
    row(kivact::view::dynamic(
        move || (page.get(), total.get()),
        move |(current, total)| {
            let p = current.min(total.saturating_sub(1)).saturating_add(1);
            let set_page = move |next: usize| {
                let before = page.get_untracked();
                if before == next - 1 {
                    return;
                }
                binding.set(next - 1);
                if page.get_untracked() != before {
                    restore_focus.set(true);
                }
            };
            let previous_page = set_page.clone();
            let mut items = vec![
                crate::tool_button("‹")
                    .name(previous_label)
                    .disabled(total == 0 || p == 1)
                    .on_press(move || previous_page(p.saturating_sub(1).max(1)))
                    .style(style().size(30, 30).border(1, ColorToken::Border))
                    .into_view(),
            ];
            let mut nums = vec![
                1,
                total.max(1),
                p.saturating_sub(1).max(1),
                p,
                (p.saturating_add(1)).min(total.max(1)),
            ];
            if total == 0 {
                nums.clear();
            }
            nums.sort_unstable();
            nums.dedup();
            let mut previous = 0;
            for n in nums {
                if previous > 0 && n - previous > 1 {
                    items.push(crate::mono("…").into_view());
                }
                let node = node_ref();
                if n == p {
                    kivact::on_mount(move || {
                        if restore_focus.get() {
                            restore_focus.set(false);
                            node.focus();
                        }
                    });
                }
                let set_page = set_page.clone();
                items.push(
                    crate::tool_button(n.to_string())
                        .node_ref(node)
                        .style(
                            style().size(30, 30).border(1, ColorToken::Border).selected(
                                style()
                                    .background(ColorToken::Accent)
                                    .color(ColorToken::AccentText),
                            ),
                        )
                        .selected(n == p)
                        .on_press(move || set_page(n))
                        .into_view(),
                );
                previous = n;
            }
            items.push(
                crate::tool_button("›")
                    .name(next_label)
                    .disabled(p >= total)
                    .on_press(move || set_page((p.saturating_add(1)).min(total.max(1))))
                    .style(style().size(30, 30).border(1, ColorToken::Border))
                    .into_view(),
            );
            items
        },
    ))
    .role(Role::Group)
    .name(message("game.pagination.page", "Pages"))
    .style(style().gap(4).align_items(Align::Center))
}
