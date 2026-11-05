//! Role: Notification history with application-owned read state.
use crate::ColorName;

use kivact::ui::{ButtonVariant, button};

use crate::{Hue, Tokens};
use kivact::draw::Shadow;
use kivact::i18n::message;
use kivact::interaction::use_popup;
use kivact::state::IntoValue;
use kivact::state::signal;
use kivact::style::use_theme;
use kivact::style::{Align, ColorToken, Justify, Position, pct, style};
use kivact::view::{IntoView, Placement, Role, for_each, overlay, show};
use kivact::view::{Node, NodeBuilder, column, icon, node, row, text};

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
pub struct NotificationMenu<V: IntoView + NodeBuilder> {
    trigger: V,
    root: Node,
    items: kivact::state::Value<Vec<NotificationItem>>,
    mark_read: kivact::state::Callback<()>,
}
#[track_caller]
pub fn notification_menu<V: IntoView + NodeBuilder>(
    trigger: V,
    items: impl IntoValue<Vec<NotificationItem>>,
) -> NotificationMenu<V> {
    NotificationMenu {
        trigger,
        root: column(()),
        items: items.into_value(),
        mark_read: kivact::state::callback(|_| ()),
    }
}
impl<V: IntoView + NodeBuilder> NotificationMenu<V> {
    pub fn on_mark_read(mut self, mark_read: impl Into<kivact::state::Callback<()>>) -> Self {
        self.mark_read = mark_read.into();
        self
    }
}
impl<V: IntoView + NodeBuilder> NodeBuilder for NotificationMenu<V> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl<V: IntoView + NodeBuilder> IntoView for NotificationMenu<V> {
    fn into_view(self) -> kivact::view::View {
        let items = self.items;
        let trigger = self.trigger;
        let mark_read = self.mark_read;
        let title = message("pop.notifications.title", "Notifications");
        let unread = message("pop.notifications.unread", "Unread notifications");
        let mark_read_label = message("pop.notifications.mark_read", "Mark all as read");
        let open = signal(false);
        let popup = use_popup(open);
        let trigger_part = popup.trigger();
        let anchor = kivact::view::node_ref();
        let read = mark_read;
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
                                    .radius(4)
                                    .background(ColorToken::Danger),
                            )
                    },
                ))
        });
        self.root
            .child((
                trigger,
                show(popup.is_open(), move || {
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
                                |item| {
                                    let initial = item.get_untracked();
                                    row((
                                        node()
                                            .child(icon(initial.icon))
                                            .style(move || {
                                                let theme = &use_theme().get();
                                                let p = theme.ext::<Tokens>();
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
                                                style()
                                                    .font_size(13.5)
                                                    .weight(500)
                                                    .padding_right(16),
                                            ),
                                            text(move || item.get().body).style(
                                                style().font_size(13).color(ColorToken::TextMuted),
                                            ),
                                            text(move || item.get().timestamp)
                                                .style(style().font_size(12))
                                                .style(style().color(ColorName::TextFaint)),
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
                                    .with_node(|root| root.role(Role::Group))
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
                                .background(ColorName::Surface)
                                .border(1, ColorName::Line)
                                .shadow(Shadow::new(0, 10, 15, ColorToken::Shadow)),
                        ),
                    )
                    .with_node(|root| root.role(Role::Group))
                    .name(title)
                    .offset(4)
                    .with_node(|root| root.part(popup.content()))
                }),
            ))
            .style(style().align_items(Align::Start))
            .into_view()
    }
}
