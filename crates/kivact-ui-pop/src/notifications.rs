//! Role: Notification history with application-owned read state.

use crate::Hue;
use crate::color;
use kivact::component;
use kivact::i18n::message;
use kivact::interaction::use_popup;
use kivact::state::{Callback, Value, callback, signal};
use kivact::style::Shadow;
use kivact::style::color as core;
use kivact::style::{Align, Justify, Position, pct, style};
use kivact::ui::{ButtonVariant, button};
use kivact::view::{
    Element, IntoView, Node, Placement, Role, column, for_each, icon, node, overlay, row, show,
    text,
};

/// One notification; its identifier survives read-state updates.
#[derive(Clone, Debug, PartialEq)]
pub struct NotificationItem {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub timestamp: String,
    pub icon: kivact::style::Icon,
    pub hue: Hue,
    pub unread: bool,
}

impl NotificationItem {
    pub fn new(
        id: u64,
        title: impl Into<String>,
        body: impl Into<String>,
        timestamp: impl Into<String>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            body: body.into(),
            timestamp: timestamp.into(),
            icon: kivact::style::Icon::named("bell"),
            hue: Hue::Gray,
            unread: false,
        }
    }
}

/// Anchored notification history. Marking read calls `on_mark_read` and
/// keeps the history open. The trigger dot and row backgrounds derive from
/// the same reactive read state.
#[component]
pub fn notification_menu<V: IntoView + Element>(
    trigger: V,
    items: Value<Vec<NotificationItem>>,
    on_mark_read: Option<Callback<()>>,
) -> impl Element {
    let title = message("pop.notifications.title", "Notifications");
    let unread = message("pop.notifications.unread", "Unread notifications");
    let mark_read_label = message("pop.notifications.mark_read", "Mark all as read");
    let open = signal(false);
    let popup = use_popup(open);
    let trigger_part = popup.trigger();
    let anchor = kivact::view::node_ref();
    let read = on_mark_read.unwrap_or_else(|| callback(|_| ()));
    let trigger = trigger.with_node(|root| {
        root.with_node(|root| root.part(trigger_part))
            .node_ref(anchor)
            .child(show(
                move || items.with(|v| v.iter().any(|item| item.unread)),
                move || {
                    node()
                        .with_node(|root| root.role(Role::Status))
                        .name(unread)
                        .style(
                            style()
                                .position(Position::Absolute)
                                .right(7)
                                .top(7)
                                .size(7, 7)
                                .border_radius(4)
                                .background(core::DANGER),
                        )
                },
            ))
    });
    column(())
        .child((
            trigger,
            show(popup.is_open(), move || {
                overlay(
                    anchor,
                    Placement::Below,
                    column((
                        row((
                            text(title).style(style().flex_grow(1).font_weight(600)),
                            button(mark_read_label)
                                .variant(ButtonVariant::Ghost)
                                .style(style().height(28).padding_x(4).font_size(12))
                                .on_press(move || read.call(())),
                        ))
                        .style(
                            style()
                                .padding_x(12)
                                .padding_y(10)
                                .align_items(Align::Center)
                                .border_bottom(1, core::BORDER),
                        ),
                        for_each(items, |item| item.id, notification_row),
                    ))
                    .style(
                        style()
                            .width(340)
                            .max_width(pct(95))
                            .padding(4)
                            .border_radius(8)
                            .clip(true),
                    )
                    .style(
                        style()
                            .background(color::SURFACE)
                            .border(1, color::LINE)
                            .shadow(Shadow::new(0, 10, 15, core::SHADOW)),
                    ),
                )
                .with_node(|root| root.role(Role::Group))
                .name(title)
                .offset(4)
                .with_node(|root| root.part(popup.content()))
            }),
        ))
        .style(style().align_items(Align::Start))
}

fn notification_row(item: Value<NotificationItem>) -> Node {
    let initial = item.get_untracked();
    row((
        node()
            .child(icon(initial.icon))
            .style(move || {
                let (background, color) = item.get().hue.colors();
                style().background(background).color(color)
            })
            .style(
                style()
                    .size(32, 32)
                    .border_radius(6)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .flex_shrink(0),
            ),
        column((
            text(move || item.get().title)
                .style(style().font_size(13.5).font_weight(500).padding_right(16)),
            text(move || item.get().body).style(style().font_size(13).color(core::TEXT_MUTED)),
            text(move || item.get().timestamp)
                .style(style().font_size(12))
                .style(style().color(color::TEXT_FAINT)),
        ))
        .style(style().flex_grow(1).gap(0)),
        show(
            move || item.get().unread,
            || {
                node().style(
                    style()
                        .position(Position::Absolute)
                        .right(10)
                        .top(14)
                        .size(6, 6)
                        .border_radius(3)
                        .background(core::ACCENT),
                )
            },
        ),
    ))
    .with_node(|root| root.role(Role::Group))
    .name(move || item.get().title)
    .selected(move || item.get().unread)
    .style(
        style()
            .padding(8)
            .gap(10)
            .align_items(Align::Start)
            .border_radius(6)
            .hover(style().background(core::SURFACE_ALT)),
    )
}
