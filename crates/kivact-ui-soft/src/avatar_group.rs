//! Role: Lay out overlapping avatars and manage their keyboard navigation.
use crate::avatar_options::{AvatarShape, AvatarSize};
use kivact::i18n::message;
use kivact::state::IntoValue;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::node;

use kivact::state::IntoText;
use kivact::state::Signal;
use kivact::state::Value;
use kivact::state::effect;
use kivact::state::on_cleanup;
use kivact::state::provide;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::style;
use kivact::view::Handled;
use kivact::view::IntoChildren;
use kivact::view::IntoView;
use kivact::view::Key;
use kivact::view::NodeRef;
use kivact::view::ReadingDirection;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::node_ref;
use kivact::view::use_direction;

#[derive(Clone)]
pub(crate) struct AvatarGroupContext {
    pub(crate) side: f64,
    pub(crate) shape: AvatarShape,
    active: Signal<Option<NodeRef>>,
    entries: Signal<Vec<(NodeRef, Value<bool>)>>,
}

impl AvatarGroupContext {
    pub(crate) fn register(&self, root: Node) -> Node {
        let reference = node_ref();
        let Self {
            active, entries, ..
        } = *self;
        let focused = reference.focused();
        entries.update(|list| list.push((reference, focused)));
        if active.get_untracked().is_none() {
            active.set(Some(reference));
        }
        effect(move || {
            if focused.get() {
                active.set(Some(reference));
            }
        });
        on_cleanup(move || {
            entries.update(|list| list.retain(|(item, _)| *item != reference));
            if active.get_untracked() == Some(reference) {
                active.set(entries.get_untracked().first().map(|(item, _)| *item));
            }
        });
        // Keep every entry pointer-focusable; arrow navigation uses the same node references.
        root.node_ref(reference).focusable(true)
    }

    fn navigate(&self, key: Key, rtl: bool) -> Handled {
        let entries = self.entries.get_untracked();
        let Some(index) = entries
            .iter()
            .position(|(_, focused)| focused.get_untracked())
        else {
            return Handled::No;
        };
        let next = match key {
            Key::Home => 0,
            Key::End => entries.len() - 1,
            Key::ArrowDown => (index + 1) % entries.len(),
            Key::ArrowUp => (index + entries.len() - 1) % entries.len(),
            Key::ArrowLeft if rtl => (index + 1) % entries.len(),
            Key::ArrowRight if !rtl => (index + 1) % entries.len(),
            Key::ArrowLeft | Key::ArrowRight => (index + entries.len() - 1) % entries.len(),
            _ => return Handled::No,
        };
        entries[next].0.focus();
        Handled::Yes
    }
}

/// A facepile with a shared size and silhouette.
pub struct AvatarGroup<C> {
    root: Node,
    children: C,
    size: Value<AvatarSize>,
    shape: Value<AvatarShape>,
}

/// Group avatars or overflow counters in an overlapping row.
pub fn avatar_group<C: IntoChildren + 'static>(children: C) -> AvatarGroup<C> {
    AvatarGroup {
        root: node()
            .with_node(move |node| node.role(Role::Group))
            .name(message("soft.avatar_group.label", "Avatars")),
        children,
        size: (AvatarSize::default()).into_value(),
        shape: (AvatarShape::default()).into_value(),
    }
}

impl<C> AvatarGroup<C> {
    /// Set the size of every avatar.
    pub fn size(mut self, size: impl IntoValue<AvatarSize>) -> Self {
        self.size = size.into_value();
        self
    }
    /// Set the silhouette of every avatar.
    pub fn shape(mut self, shape: impl IntoValue<AvatarShape>) -> Self {
        self.shape = shape.into_value();
        self
    }
    /// Set the group's accessible label.
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.root = self.root.name(label);
        self
    }
}

impl<C> NodeBuilder for AvatarGroup<C> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<C: IntoChildren + 'static> IntoView for AvatarGroup<C> {
    fn into_view(self) -> View {
        let Self {
            mut root,
            children,
            size,
            shape,
        } = self;
        let rtl = use_direction().get() == ReadingDirection::Rtl;
        let side = size.get().side();
        let entries = signal(Vec::new());
        let context = AvatarGroupContext {
            side,
            shape: shape.get(),
            entries,
            active: signal(None),
        };
        let keyboard = context.clone();
        if !root.has_description() {
            let hint = message(
                "soft.avatar_group.keyboard_hint",
                "Use arrow keys to move between avatars",
            );
            root = root.description(move || {
                if entries.get().is_empty() {
                    String::new()
                } else {
                    hint.get()
                }
            });
        }
        root.with_node(move |node| {
            node.base_style(
                style()
                    .direction(if rtl {
                        Direction::RowReverse
                    } else {
                        Direction::Row
                    })
                    .align_items(Align::Center)
                    .shrink(0),
            )
        })
        .on_key(move |event| keyboard.navigate(event.key(), rtl))
        .child(provide(context, move || {
            let children = children.into_views();
            let overlap = if children.is_empty() {
                0.0
            } else {
                (side * 0.25).round()
            };
            let look = style()
                .direction(if rtl {
                    Direction::RowReverse
                } else {
                    Direction::Row
                })
                .align_items(Align::Center);
            node()
                .style(if rtl {
                    look.padding_right(overlap)
                } else {
                    look.padding_left(overlap)
                })
                .child(children)
        }))
        .into_view()
    }
}
