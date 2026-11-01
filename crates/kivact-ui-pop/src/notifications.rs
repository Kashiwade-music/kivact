//! Role: Notification history with application-owned read state.
use crate::Color;
use kivact::primitive::NodeExt;
use kivact::ui::{ButtonVariant, button};

use crate::{Hue, Tokens};
use kivact::control::{Node, NodeBuilder, column, icon, node, row, text};
use kivact::draw::Shadow;
use kivact::i18n::message;
use kivact::interaction::use_popup;
use kivact::signal;
use kivact::style::current_theme;
use kivact::style::{Align, ColorToken, Justify, Position, pct, style};
use kivact::view::{IntoValue, IntoView, Placement, Role, callback, for_each, overlay, show};

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

/// Anchored notification history. Marking read calls the owner and keeps it open.
/// The trigger dot and row backgrounds derive from the same reactive read state.
#[track_caller]
pub fn notification_menu<V, F, R>(
    mut trigger: V,
    items: impl IntoValue<Vec<NotificationItem>>,
    mark_read: F,
) -> Node
where
    V: IntoView + NodeBuilder,
    F: Fn() -> R + 'static,
    R: kivact::view::HandlerResult,
{
    let items = items.into_value();
    let title = message("pop.notifications.title", "Notifications");
    let unread = message("pop.notifications.unread", "Unread notifications");
    let mark_read_label = message("pop.notifications.mark_read", "Mark all as read");
    let open = signal(false);
    let popup = use_popup(open);
    let trigger_part = popup.trigger();
    let anchor = trigger_part.node_ref();
    let read = callback(move |()| mark_read());
    let root = trigger.node_mut();
    *root = std::mem::replace(root, node())
        .part(trigger_part)
        .child(show(
            move || items.with(|v| v.iter().any(|item| item.unread)),
            move || {
                node().role(Role::Status).name(unread).style(
                    style()
                        .position(Position::Absolute)
                        .right(7)
                        .top(7)
                        .size(7, 7)
                        .radius(4)
                        .background(ColorToken::Danger),
                )
            },
        ));
    column((
        trigger,
        show(popup.shown(), move || {
            overlay(
                anchor,
                Placement::Below,
                column((
                    row((
                        text(title).style(style().grow(1).weight(600)),
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
                            .border_bottom(1, ColorToken::Border),
                    ),
                    for_each(
                        items,
                        |item| item.id,
                        |_, item| {
                            let initial = item.get_untracked();
                            row((
                                node()
                                    .child(icon(initial.icon))
                                    .style(move || {
                                        let theme = &current_theme();
                                        let p = theme.get::<Tokens>();
                                        let (background, color) = p.hue(item.get().hue);
                                        style().background(background).color(color)
                                    })
                                    .style(
                                        style()
                                            .size(32, 32)
                                            .radius(6)
                                            .align_items(Align::Center)
                                            .justify_content(Justify::Center)
                                            .shrink(0),
                                    ),
                                column((
                                    text(move || item.get().title).style(
                                        style().font_size(13.5).weight(500).padding_right(16),
                                    ),
                                    text(move || item.get().body)
                                        .style(style().font_size(13).color(ColorToken::TextMuted)),
                                    text(move || item.get().timestamp)
                                        .style(style().font_size(12))
                                        .style(style().color(Color::TextFaint)),
                                ))
                                .style(style().grow(1).gap(0)),
                                show(
                                    move || item.get().unread,
                                    || {
                                        node().style(
                                            style()
                                                .position(Position::Absolute)
                                                .right(10)
                                                .top(14)
                                                .size(6, 6)
                                                .radius(3)
                                                .background(ColorToken::Accent),
                                        )
                                    },
                                ),
                            ))
                            .role(Role::Group)
                            .name(move || item.get().title)
                            .selected(move || item.get().unread)
                            .style(
                                style()
                                    .padding(8)
                                    .gap(10)
                                    .align_items(Align::Start)
                                    .radius(6)
                                    .hover(style().background(ColorToken::SurfaceAlt)),
                            )
                        },
                    ),
                ))
                .style(
                    style()
                        .width(340)
                        .max_width(pct(95))
                        .padding(4)
                        .radius(8)
                        .clip(true),
                )
                .style(
                    style()
                        .background(Color::Surface)
                        .border(1, Color::Line)
                        .shadow(Shadow::new(0, 10, 15, ColorToken::Shadow)),
                ),
            )
            .role(Role::Group)
            .name(title)
            .offset(4)
            .part(popup.content())
        }),
    ))
    .style(style().align_items(Align::Start))
}
