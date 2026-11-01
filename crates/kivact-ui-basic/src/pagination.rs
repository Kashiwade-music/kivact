//! Role: The Lyra pagination bar with page links, previous/next and an ellipsis.

use crate::button::{Look, button_style};
use crate::parts::pressable;
use crate::theme_access::tokens;
use kivact::control::icon;
use kivact::control::{Node, NodeBuilder, node, row, text};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Cursor, Direction, Justify, pct, style};
use kivact::ui::ControlSize;
use kivact::view::HandlerResult;
use kivact::view::{IntoText, IntoValue, IntoView, Role, View, callback, for_each};
use kivact::view::{ReadingDirection, current_direction};
use kivact::{Binding, Callback, Value, memo};
use std::rc::Rc;

/// A pagination bar (pagination.tsx): a `Role::Navigation` landmark named
/// "Pagination" (the `pagination.label` message).
///
/// Pass [`pagination_link`], [`pagination_previous`], [`pagination_next`]
/// and [`pagination_ellipsis`] as children, or use [`pagination`] for
/// a complete bar. In a right-to-left [`direction`](kivact::view::provide_direction) the
/// bar runs from the right and the chevrons mirror.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn bar() -> impl IntoView {
///     basic::pagination_content((
///         basic::pagination_previous(),
///         basic::pagination_link("1"),
///         basic::pagination_link("2").active(true),
///         basic::pagination_ellipsis(),
///         basic::pagination_next(),
///     ))
/// }
/// ```
#[track_caller]
pub fn pagination_content(children: impl IntoView) -> Node {
    bar().child(track(children))
}

/// The landmark around the links.
#[track_caller]
fn bar() -> Node {
    node()
        .role(Role::Navigation)
        .name(kivact::i18n::message(
            "basic.pagination.label",
            "Pagination",
        ))
        .style(
            style()
                .direction(Direction::Row)
                .width(pct(100))
                .justify_content(Justify::Center),
        )
}

/// The row of links, reversed in a right-to-left direction.
#[track_caller]
fn track(children: impl IntoView) -> Node {
    let rtl = current_direction() == ReadingDirection::Rtl;
    row(children).style(style().align_items(Align::Center).gap(2).direction(if rtl {
        Direction::RowReverse
    } else {
        Direction::Row
    }))
}

/// Where a [`PaginationLink`] shows its chevron.
#[derive(Clone, Copy, PartialEq, Eq)]
enum LinkKind {
    Page,
    Previous,
    Next,
}

/// A page link, or the previous and next links. Created by [`pagination_link`].
pub struct PaginationLink {
    root: Node,
    kind: LinkKind,
    shown: Option<Value<String>>,
    active: Value<bool>,
    on_press: Option<Callback<()>>,
}

/// A link of `kind` whose accessible name is `label` and whose text is `shown`.
#[track_caller]
fn link(kind: LinkKind, label: Value<String>, shown: Option<Value<String>>) -> PaginationLink {
    let active = false.into_value();
    PaginationLink {
        root: node().name(label).selected(active),
        kind,
        shown,
        active,
        on_press: None,
    }
}

/// A link showing `text` (usually a page number), square like an icon button.
///
/// The active page is outlined; the others are ghost buttons.
#[track_caller]
pub fn pagination_link(text_value: impl IntoText) -> PaginationLink {
    let label = text_value.into_text_value();
    link(LinkKind::Page, label, Some(label))
}

/// The "Previous" link: a left chevron and text, named "Go to previous page".
#[track_caller]
pub fn pagination_previous() -> PaginationLink {
    link(
        LinkKind::Previous,
        kivact::i18n::message("basic.pagination.previous_page", "Go to previous page"),
        Some(kivact::i18n::message(
            "basic.pagination.previous",
            "Previous",
        )),
    )
}

/// The "Next" link: text and a right chevron, named "Go to next page".
#[track_caller]
pub fn pagination_next() -> PaginationLink {
    link(
        LinkKind::Next,
        kivact::i18n::message("basic.pagination.next_page", "Go to next page"),
        Some(kivact::i18n::message("basic.pagination.next", "Next")),
    )
}

impl PaginationLink {
    /// Mark the link as the current page: outlined and `selected`.
    pub fn active(mut self, active: impl IntoValue<bool>) -> Self {
        let active = active.into_value();
        self.active = active;
        self.root = self.root.selected(active);
        self
    }

    /// Report presses (click, Enter, Space) to `f`.
    pub fn on_press<R: HandlerResult>(mut self, f: impl Fn() -> R + 'static) -> Self {
        self.on_press = Some(callback(move |_| f()));
        self
    }
}

impl NodeBuilder for PaginationLink {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for PaginationLink {
    fn into_view(self) -> View {
        let rtl = current_direction() == ReadingDirection::Rtl;
        let kind = self.kind;
        let (root, state) = pressable(
            self.root,
            Role::Link,
            Some(self.on_press.unwrap_or_else(|| callback(|()| {}))),
        );
        let active = self.active;
        let root = root.base_style(move || {
            let p = tokens();
            let look = if active.get() {
                Look::Outline
            } else {
                Look::Ghost
            };
            let base = button_style(&p, look, ControlSize::Medium, &state.get());
            let tweak = match kind {
                LinkKind::Page => style().padding_x(0).min_width(32),
                LinkKind::Previous => style().padding_left(6),
                LinkKind::Next => style().padding_right(6),
            };
            base.merge(&tweak).merge(&style().cursor(Cursor::Pointer))
        });
        let (before, after) = match (kind, rtl) {
            (LinkKind::Previous, false) => (Some("chevron-left"), None),
            (LinkKind::Previous, true) => (Some("chevron-right"), None),
            (LinkKind::Next, false) => (None, Some("chevron-right")),
            (LinkKind::Next, true) => (None, Some("chevron-left")),
            (LinkKind::Page, _) => (None, None),
        };
        root.child(before.map(icon))
            .child(self.shown.map(text))
            .child(after.map(icon))
            .into_view()
    }
}

/// The "…" between page links, read as the text "More pages".
#[track_caller]
pub fn pagination_ellipsis() -> Node {
    node()
        .role(Role::Text)
        .name(kivact::i18n::message("basic.pagination.more", "More pages"))
        .style(
            style()
                .size(32, 32)
                .shrink(0)
                .direction(Direction::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center),
        )
        .child(icon("ellipsis"))
}

/// One cell of the bar [`pagination`] builds; the value is its key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Cell {
    Previous,
    Page(usize),
    Gap(usize),
    Next,
}

/// The cells for zero-based `page` of `count` pages: first, last, the
/// current page and its neighbours, with a gap wherever pages are skipped.
fn cells(page: usize, count: usize) -> Vec<Cell> {
    let mut out = vec![Cell::Previous];
    let mut shown: Vec<usize> = [
        0,
        page.saturating_sub(1),
        page,
        page.saturating_add(1),
        count.wrapping_sub(1),
    ]
    .into_iter()
    .filter(|n| *n < count)
    .collect();
    shown.sort_unstable();
    shown.dedup();
    let mut last = None;
    for n in shown {
        if let Some(before) = last
            && n > before + 1
        {
            out.push(Cell::Gap(before));
        }
        out.push(Cell::Page(n));
        last = Some(n);
    }
    out.push(Cell::Next);
    out
}

/// A complete bar for zero-based `page` of `count` pages. Created by [`pagination`].
pub struct PaginationPages {
    root: Node,
    page: Binding<usize>,
    count: Value<usize>,
    format: Rc<dyn Fn(usize) -> String>,
}

/// A complete bar for zero-based `page` of `count` pages.
///
/// It shows the first page, the last page, the current page and its
/// neighbours, "…" where pages are skipped, and previous/next links that are
/// disabled at the ends. Presses write the new page to the binding. The page
/// links read "1", "2", ...; [`PaginationPages::label_format`] changes that.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn pages() -> impl IntoView {
///     let page = signal(0usize);
///     basic::pagination(page, 12usize).label_format(|number| format!("{number}"))
/// }
/// ```
#[track_caller]
pub fn pagination(
    page: impl Into<Binding<usize>>,
    count: impl IntoValue<usize>,
) -> PaginationPages {
    PaginationPages {
        root: bar(),
        page: page.into(),
        count: count.into_value(),
        format: Rc::new(|number| number.to_string()),
    }
}

impl PaginationPages {
    /// Write the page links with `format`, which gets the one-based page number
    /// (1 for the first page). It also names the link for assistive technology.
    pub fn label_format(mut self, format: impl Fn(usize) -> String + 'static) -> Self {
        self.format = Rc::new(format);
        self
    }
}

impl NodeBuilder for PaginationPages {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for PaginationPages {
    fn into_view(self) -> View {
        let Self {
            root,
            page,
            count,
            format,
        } = self;
        let effective = memo(move || page.get().min(count.get().saturating_sub(1)));
        let items = memo(move || cells(effective.get(), count.get()));
        let links = for_each(
            items,
            |cell| *cell,
            move |cell, _| match cell {
                Cell::Previous => pagination_previous()
                    .disabled(move || count.get() == 0 || effective.get() == 0)
                    .on_press(move || page.set(effective.get().saturating_sub(1)))
                    .into_view(),
                Cell::Next => pagination_next()
                    .disabled(move || effective.get().saturating_add(1) >= count.get())
                    .on_press(move || {
                        let last = count.get().saturating_sub(1);
                        page.set((effective.get().saturating_add(1)).min(last));
                    })
                    .into_view(),
                Cell::Page(n) => pagination_link(format(n + 1))
                    .active(move || effective.get() == n)
                    .on_press(move || page.set(n))
                    .into_view(),
                Cell::Gap(_) => pagination_ellipsis().into_view(),
            },
        );
        root.child(track(links)).into_view()
    }
}
