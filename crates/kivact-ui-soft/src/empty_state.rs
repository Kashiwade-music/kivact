//! Role: Present a centered empty-state title, description, and actions.

use crate::{TextColor, TextJustify, TextKind, text, tokens};
use kivact::control::{Node, NodeBuilder, column, node};
use kivact::interaction::announcer;
use kivact::primitive::{NodeExt, NodeRefExt, part};
use kivact::style::{Align, Direction, Justify, pct, style};
use kivact::view::{
    IntoText, IntoValue, IntoView, LiveRegion, ReadingDirection, View, current_direction, node_ref,
};
use kivact::{Value, effect, on_mount, signal, var};

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
        let notices = announcer();
        let source = node_ref();
        let mounted = signal(None::<bool>);
        let first = var(true);
        let spoken = var(String::new());
        let has_notice = var(false);
        effect(move || {
            let Some(initial) = mounted.get() else {
                return;
            };
            let initial = first.update(|first| std::mem::replace(first, false)) && initial;
            let visible = source.is_visible();
            if !visible {
                spoken.set(String::new());
            }
            let eligible = visible && !source.is_in_live_region();
            let message = if eligible {
                announcement_text(&title.get(), description.map(|text| text.get()).as_deref())
            } else {
                String::new()
            };
            if message.is_empty() {
                if has_notice.update(|has_notice| std::mem::replace(has_notice, false)) {
                    notices.announce("", LiveRegion::Polite);
                }
            } else if spoken.with(|spoken| *spoken != message) {
                spoken.set(message.clone());
                if !initial {
                    notices.announce(message, LiveRegion::Polite);
                    has_notice.set(true);
                }
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
        .node_ref(source)
        .part(part(|input| input).on_bind(move |_| {
            let initial = kivact::platform::is_initial_render();
            // Establish the empty live region before placing any text in it.
            on_mount(move || mounted.set(Some(initial)));
        }))
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
        .child(notices.outlet())
        .into_view()
    }
}

fn announcement_text(title: &str, description: Option<&str>) -> String {
    let normalize = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
    let title = normalize(title);
    let description = description.map(normalize).unwrap_or_default();
    if title.is_empty() {
        description
    } else if description.is_empty() {
        title
    } else if title.ends_with(['.', '!', '?']) {
        format!("{title} {description}")
    } else {
        format!("{title}. {description}")
    }
}
