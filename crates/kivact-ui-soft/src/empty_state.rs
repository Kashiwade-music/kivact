//! Role: Present a centered empty-state title, description, and actions.

use crate::{TextColor, TextJustify, TextKind, text, tokens, visually_hidden};
use kivact::control::{Node, NodeBuilder, column, node};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Direction, Justify, pct, style};
use kivact::view::{
    IntoText, IntoValue, IntoView, LiveRegion, ReadingDirection, View, current_direction,
};
use kivact::{Value, effect, signal};
use std::cell::Cell;

/// A centered message for a region with no content.
pub struct EmptyState {
    root: Node,
    title: Value<String>,
    description: Option<Value<String>>,
    icon: Option<Box<dyn FnOnce() -> View>>,
    actions: Option<Box<dyn FnOnce() -> View>>,
    heading_level: u32,
    compact: Value<bool>,
}

/// Create an empty state with a semantic heading.
pub fn empty_state(title: impl IntoText) -> EmptyState {
    EmptyState {
        root: node(),
        title: title.into_text_value(),
        description: None,
        icon: None,
        actions: None,
        heading_level: 3,
        compact: false.into_value(),
    }
}

impl EmptyState {
    /// Set the supporting description.
    pub fn description(mut self, description: impl IntoText) -> Self {
        self.description = Some(description.into_text_value());
        self
    }

    /// Add a decorative visual above the title.
    pub fn icon<V: IntoView>(mut self, icon: impl FnOnce() -> V + 'static) -> Self {
        self.icon = Some(Box::new(move || icon().into_view()));
        self
    }

    /// Add actions below the message.
    pub fn actions<V: IntoView>(mut self, actions: impl FnOnce() -> V + 'static) -> Self {
        self.actions = Some(Box::new(move || actions().into_view()));
        self
    }

    /// Override the heading's accessibility level.
    pub fn heading_level(mut self, level: u32) -> Self {
        self.heading_level = level.clamp(1, 6);
        self
    }

    /// Use compact spacing and stack the actions.
    pub fn compact(mut self, compact: impl IntoValue<bool>) -> Self {
        self.compact = compact.into_value();
        self
    }
}

impl NodeBuilder for EmptyState {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for EmptyState {
    fn into_view(self) -> View {
        let Self {
            root,
            title,
            description,
            icon,
            actions,
            heading_level,
            compact,
        } = self;
        let announcement = signal(String::new());
        let initial = Cell::new(true);
        // TODO(soft-core: C-19): distinguish late mounts and detect ancestor live regions.
        effect(move || {
            let title = title.get();
            let description = description.map(|value| value.get());
            if !initial.replace(false) {
                announcement.set(match description {
                    Some(description) if !description.is_empty() => {
                        format!("{title}. {description}")
                    }
                    _ => title,
                });
            }
        });
        let text_group = column((
            text(title)
                .kind(TextKind::Large)
                .justify(TextJustify::Center)
                .heading_level(heading_level)
                .style(move || {
                    if compact.get() {
                        style().font_size(tokens().typography.size_md)
                    } else {
                        style()
                    }
                }),
            description.map(|description| {
                text(description)
                    .color(TextColor::Secondary)
                    .justify(TextJustify::Center)
                    .style(move || {
                        if compact.get() {
                            style().font_size(tokens().typography.size_sm)
                        } else {
                            style()
                        }
                    })
            }),
        ))
        .style(
            style()
                .align_items(Align::Center)
                .max_width(360)
                .min_width(0),
        );

        let actions = actions.map(|actions| {
            node()
                .style(move || {
                    let spacing = &tokens().spacing;
                    let direction = if compact.get() {
                        Direction::Column
                    } else if current_direction() == ReadingDirection::Rtl {
                        Direction::RowReverse
                    } else {
                        Direction::Row
                    };
                    style()
                        .direction(direction)
                        .align_items(Align::Center)
                        .justify_content(Justify::Center)
                        .gap(spacing.s2)
                        .margin_top(spacing.s1)
                })
                .child(actions())
        });

        root.base_style(move || {
            let spacing = &tokens().spacing;
            let (gap, padding_x, padding_y) = if compact.get() {
                (spacing.s2, spacing.s4, spacing.s4)
            } else {
                (spacing.s4, spacing.s6, spacing.s8)
            };
            style()
                .direction(Direction::Column)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .width(pct(100))
                .min_width(0)
                .gap(gap)
                .padding_x(padding_x)
                .padding_y(padding_y)
        })
        .child(icon.map(|icon| icon()))
        .child(text_group)
        .child(actions)
        .child(visually_hidden(announcement).live(LiveRegion::Polite))
        .into_view()
    }
}
