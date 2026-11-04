//! Role: The Lyra card with header, image, content and footer slots.

use crate::Color;
use crate::theme_access::tokens;
use kivact::state::Value;
use kivact::view::{Node, NodeBuilder, column, node, row, text};

use kivact::state::{IntoText, IntoValue};
use kivact::style::{Align, Direction, Overflow, style};
use kivact::view::{IntoView, View};

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
    root: Node,
    title: Value<String>,
    description: Option<Value<String>>,
    action: Option<kivact::view::View>,
    bordered: Value<bool>,
}

/// A card header titled `title`.
#[track_caller]
pub fn card_header(title: impl IntoText) -> CardHeader {
    CardHeader {
        root: node(),
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
    pub fn action(mut self, action: impl IntoView) -> Self {
        self.action = Some(action.into_view());
        self
    }

    /// Draw a rule under the header (`border-b`) while true (constant or
    /// reactive).
    #[track_caller]
    pub fn bordered(mut self, bordered: impl IntoValue<bool>) -> Self {
        self.bordered = bordered.into_value();
        self
    }

    fn build(self, spacing: Value<f64>) -> Node {
        let titles = column((
            text(self.title).style(style().font_size(14).line_height(20.0 / 14.0).weight(500)),
            self.description.map(|description| {
                text(description).style(style().line_height(1.625).color(Color::MutedForeground))
            }),
        ))
        .style(style().gap(4).grow(1).min_width(0));
        let bordered = self.bordered;
        self.root
            .child((
                titles,
                self.action
                    .map(|action| row(action).style(style().align_self(Align::Start).shrink(0))),
            ))
            .base_style(move || {
                let s = style()
                    .align_items(Align::Start)
                    .gap(8)
                    .padding_x(spacing.get());
                if bordered.get() {
                    s.padding_bottom(spacing.get())
                        .border_bottom(1, tokens().border)
                } else {
                    s
                }
            })
    }
}

/// A card (card.tsx). Created by [`card`].
pub struct Card {
    root: Node,
    size: kivact::state::Value<CardSize>,
    header: Option<View>,
    image: Option<kivact::view::View>,
    content: Option<kivact::view::View>,
    footer: Option<kivact::view::View>,
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
///         .content(text_input("Email", signal(String::new())))
///         .footer(button("Login"))
/// }
/// ```
#[track_caller]
pub fn card() -> Card {
    Card {
        root: node(),
        size: CardSize::Default.into_value(),
        header: None,
        image: None,
        content: None,
        footer: None,
    }
}

impl Card {
    /// Choose the spacing.
    pub fn size(mut self, size: impl kivact::state::IntoValue<CardSize>) -> Self {
        self.size = size.into_value();
        self
    }

    /// Set the header.
    pub fn header(mut self, header: impl IntoView) -> Self {
        self.header = Some(header.into_view());
        self
    }

    /// Show `image` at the top edge (full width, no top padding).
    pub fn image(mut self, image: impl IntoView) -> Self {
        self.image = Some(image.into_view());
        self
    }

    /// Set the content, inset by the card spacing.
    pub fn content(mut self, content: impl IntoView) -> Self {
        self.content = Some(content.into_view());
        self
    }

    /// Set the footer: a row under a rule, with the card spacing around it.
    pub fn footer(mut self, footer: impl IntoView) -> Self {
        self.footer = Some(footer.into_view());
        self
    }
}

impl NodeBuilder for Card {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Card {
    fn into_view(self) -> View {
        let size = self.size;
        let spacing = kivact::state::memo(move || size.get().spacing());
        let has_footer = self.footer.is_some();
        let has_image = self.image.is_some();
        let header = self
            .header
            .map(|header| kivact::state::provide(CardSpacing(spacing), move || header));
        let content = self.content.map(|content| {
            // `text-xs/relaxed` of the card; the header and footer keep their
            // own leading so a button's label stays centered.
            column(content).style(move || style().padding_x(spacing.get()).line_height(1.625))
        });
        let footer = self.footer.map(|footer| {
            row(footer).style(move || {
                style()
                    .align_items(Align::Center)
                    .padding(spacing.get())
                    .border_top(1, Color::Border)
            })
        });
        self.root
            .base_style(move || {
                let p = tokens();
                style()
                    .direction(Direction::Column)
                    .gap(spacing.get())
                    .padding_top(if has_image { 0.0 } else { spacing.get() })
                    .padding_bottom(if has_footer { 0.0 } else { spacing.get() })
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

kivact::state::constant_value!(CardSize);

#[derive(Clone, Copy)]
struct CardSpacing(Value<f64>);
impl NodeBuilder for CardHeader {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for CardHeader {
    fn into_view(self) -> View {
        kivact::state::provide((), move || {
            let spacing = kivact::state::try_use_context::<CardSpacing>()
                .map(|spacing| spacing.get().0)
                .unwrap_or_else(|| 16.0.into_value());
            self.build(spacing)
        })
    }
}
