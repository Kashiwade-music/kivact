//! Role: The Lyra card with header, image, content and footer slots.

use crate::Color;
use crate::slot::{Slot, slot};
use crate::theme_access::tokens;
use kivact::Value;
use kivact::control::{Node, NodeBuilder, column, node, row, text};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Direction, Overflow, style};
use kivact::view::{IntoText, IntoValue, IntoView, View};

/// The spacing of a [`Card`] (card.tsx `size`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CardSize {
    /// 16 px spacing.
    #[default]
    Default,
    /// 12 px spacing.
    Sm,
}

impl CardSize {
    fn spacing(self) -> f64 {
        match self {
            CardSize::Default => 16.0,
            CardSize::Sm => 12.0,
        }
    }
}

/// The header of a [`Card`]: title, description and an action. Created by [`card_header`].
pub struct CardHeader {
    title: Value<String>,
    description: Option<Value<String>>,
    action: Option<Slot>,
    bordered: Value<bool>,
}

/// A card header titled `title`.
#[track_caller]
pub fn card_header(title: impl IntoText) -> CardHeader {
    CardHeader {
        title: title.into_text_value(),
        description: None,
        action: None,
        bordered: false.into_value(),
    }
}

impl CardHeader {
    /// Show `text` under the title.
    pub fn description(mut self, text: impl IntoText) -> Self {
        self.description = Some(text.into_text_value());
        self
    }

    /// Show `action` to the right of the title.
    pub fn action<V: IntoView>(mut self, action: impl FnOnce() -> V + 'static) -> Self {
        self.action = Some(slot(action));
        self
    }

    /// Draw a rule under the header (`border-b`) while true (constant or
    /// reactive).
    #[track_caller]
    pub fn bordered(mut self, bordered: impl IntoValue<bool>) -> Self {
        self.bordered = bordered.into_value();
        self
    }

    fn build(self, spacing: f64) -> Node {
        let titles = column((
            text(self.title).style(style().font_size(14).line_height(20.0 / 14.0).weight(500)),
            self.description.map(|description| {
                text(description).style(style().line_height(1.625).color(Color::MutedForeground))
            }),
        ))
        .style(style().gap(4).grow(1).min_width(0));
        let bordered = self.bordered;
        row((
            titles,
            self.action
                .map(|action| row(action).style(style().align_self(Align::Start).shrink(0))),
        ))
        .style(move || {
            let s = style().align_items(Align::Start).gap(8).padding_x(spacing);
            if bordered.get() {
                s.padding_bottom(spacing).border_bottom(1, tokens().border)
            } else {
                s
            }
        })
    }
}

/// A card (card.tsx). Created by [`card`].
pub struct Card {
    root: Node,
    size: CardSize,
    header: Option<CardHeader>,
    image: Option<Slot>,
    content: Option<Slot>,
    footer: Option<Slot>,
}

/// An empty card; fill it with slots.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn login() -> impl IntoView {
///     basic::card()
///         .header(basic::card_header("Login to your account")
///             .description("Enter your email below to login"))
///         .content(|| text_input("Email", signal(String::new())))
///         .footer(|| button("Login"))
/// }
/// ```
#[track_caller]
pub fn card() -> Card {
    Card {
        root: node(),
        size: CardSize::Default,
        header: None,
        image: None,
        content: None,
        footer: None,
    }
}

impl Card {
    /// Choose the spacing.
    pub fn size(mut self, size: CardSize) -> Self {
        self.size = size;
        self
    }

    /// Set the header.
    pub fn header(mut self, header: CardHeader) -> Self {
        self.header = Some(header);
        self
    }

    /// Show `image` at the top edge (full width, no top padding).
    pub fn image<V: IntoView>(mut self, image: impl FnOnce() -> V + 'static) -> Self {
        self.image = Some(slot(image));
        self
    }

    /// Set the content, inset by the card spacing.
    pub fn content<V: IntoView>(mut self, content: impl FnOnce() -> V + 'static) -> Self {
        self.content = Some(slot(content));
        self
    }

    /// Set the footer: a row under a rule, with the card spacing around it.
    pub fn footer<V: IntoView>(mut self, footer: impl FnOnce() -> V + 'static) -> Self {
        self.footer = Some(slot(footer));
        self
    }
}

impl NodeBuilder for Card {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Card {
    fn into_view(self) -> View {
        let spacing = self.size.spacing();
        let has_footer = self.footer.is_some();
        let has_image = self.image.is_some();
        let header = self.header.map(|header| header.build(spacing));
        let content = self.content.map(|content| {
            // `text-xs/relaxed` of the card; the header and footer keep their
            // own leading so a button's label stays centered.
            column(content).style(style().padding_x(spacing).line_height(1.625))
        });
        let footer = self.footer.map(|footer| {
            row(footer).style(
                style()
                    .align_items(Align::Center)
                    .padding(spacing)
                    .border_top(1, Color::Border),
            )
        });
        self.root
            .base_style(move || {
                let p = tokens();
                style()
                    .direction(Direction::Column)
                    .gap(spacing)
                    .padding_top(if has_image { 0.0 } else { spacing })
                    .padding_bottom(if has_footer { 0.0 } else { spacing })
                    .radius(0)
                    .background(p.card)
                    .color(p.card_foreground)
                    .font_size(12)
                    .ring(1, p.foreground.with_alpha(0.1))
                    .overflow(Overflow::Hidden)
            })
            .child(self.image)
            .child(header)
            .child(content)
            .child(footer)
            .into_view()
    }
}
