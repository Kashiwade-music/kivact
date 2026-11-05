//! Role: The Lyra item row with media, text, actions, group and separator.

use crate::parts::pressable;
use crate::theme_access::wide_focus_ring;
use crate::tokens::Tokens;
use kivact::state::IntoValue;
use kivact::view::{Node, NodeBuilder, column, node, row, text};

use kivact::state::IntoText;
use kivact::state::{Callback, Value};
use kivact::style::{Align, Color, Cursor, Direction, Justify, Overflow, pct, style};
use kivact::view::{IntoView, Orientation, Role, View};
use kivact::view::{ReadingDirection, use_direction};

/// The frame of an [`Item`] (item.tsx `variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ItemVariant {
    /// Transparent border.
    #[default]
    Default,
    /// Bordered.
    Outline,
    /// Muted fill.
    Muted,
}

/// The density of an [`Item`] (item.tsx `size`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ItemSize {
    /// 12 px / 10 px padding.
    #[default]
    Default,
    /// The same padding with a smaller image.
    Sm,
}

/// The frame of the media slot of an [`Item`] (item.tsx media `variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ItemMediaVariant {
    /// Bare content.
    #[default]
    Default,
    /// An icon.
    Icon,
    /// A 40 px (32 px for `Sm`) clipped picture.
    Image,
}

/// A list row (item.tsx). Created by [`item`].
pub struct Item {
    root: Node,
    /// The reading direction when the item was made.
    rtl: bool,
    title: Value<String>,
    variant: ItemVariant,
    size: kivact::state::Value<ItemSize>,
    media: Option<kivact::view::View>,
    media_variant: ItemMediaVariant,
    description: Option<Value<String>>,
    actions: Option<kivact::view::View>,
    header: Option<kivact::view::View>,
    footer: Option<kivact::view::View>,
    on_press: Option<Callback<()>>,
}

/// An item titled `title`.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn row_item() -> impl IntoView {
///     basic::item("Basic Item")
///         .variant(basic::ItemVariant::Outline)
///         .description("A simple item with title and description.")
///         .actions(button("Action"))
/// }
/// ```
#[track_caller]
pub fn item(title: impl IntoText) -> Item {
    let rtl = use_direction().get() == ReadingDirection::Rtl;
    let title = title.into_text_value();
    Item {
        root: node()
            .with_node(|root| root.role(Role::ListItem))
            .name(title),
        rtl,
        title,
        variant: ItemVariant::Default,
        size: ItemSize::Default.into_value(),
        media: None,
        media_variant: ItemMediaVariant::Default,
        description: None,
        actions: None,
        header: None,
        footer: None,
        on_press: None,
    }
}

impl Item {
    /// Choose the frame.
    pub fn variant(mut self, variant: ItemVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Choose the density.
    pub fn size(mut self, size: impl kivact::state::IntoValue<ItemSize>) -> Self {
        self.size = size.into_value();
        self
    }

    /// Show `media` before the text.
    pub fn media(mut self, media: impl IntoView) -> Self {
        self.media = Some(media.into_view());
        self
    }

    /// Choose the frame of the media.
    pub fn media_variant(mut self, variant: ItemMediaVariant) -> Self {
        self.media_variant = variant;
        self
    }

    /// Show `text` under the title (at most two lines).
    pub fn description(mut self, text: impl IntoText) -> Self {
        self.description = Some(text.into_text_value());
        self
    }

    /// Show `actions` at the end of the row.
    pub fn actions(mut self, actions: impl IntoView) -> Self {
        self.actions = Some(actions.into_view());
        self
    }

    /// Show `header` on its own line above the row.
    pub fn header(mut self, header: impl IntoView) -> Self {
        self.header = Some(header.into_view());
        self
    }

    /// Show `footer` on its own line below the row.
    pub fn footer(mut self, footer: impl IntoView) -> Self {
        self.footer = Some(footer.into_view());
        self
    }

    /// Make the whole item a button that reports presses to `f`; it highlights on hover.
    pub fn on_press(mut self, f: impl Into<kivact::state::Callback<()>>) -> Self {
        self.on_press = Some(f.into());
        self
    }
}

impl NodeBuilder for Item {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

fn frame(p: &Tokens, variant: ItemVariant) -> (Color, Color) {
    match variant {
        ItemVariant::Default => (Color::TRANSPARENT, Color::TRANSPARENT),
        ItemVariant::Outline => (Color::TRANSPARENT, p.border),
        ItemVariant::Muted => (p.muted.with_alpha(0.5), Color::TRANSPARENT),
    }
}

impl IntoView for Item {
    fn into_view(self) -> View {
        let variant = self.variant;
        let size = self.size;
        let actionable = self.on_press.is_some();
        let has_description = self.description.is_some();
        let media_variant = self.media_variant;
        let (root, _) = pressable(self.root, Role::Button, self.on_press);
        let rtl = self.rtl;
        let root = root.base_style(move || {
            let p = kivact::style::use_tokens::<crate::Tokens>();
            let (fill, line) = frame(&p, variant);
            let s = style()
                .direction(if rtl {
                    Direction::RowReverse
                } else {
                    Direction::Row
                })
                .wrap(true)
                .align_items(Align::Center)
                .width(pct(100))
                .gap(10)
                .padding_x(12)
                .padding_y(10)
                .border(1, line)
                .radius(0)
                .background(fill)
                .color(p.foreground)
                .font_size(12);
            if actionable {
                s.cursor(Cursor::Pointer)
                    .hover(style().background(p.muted))
                    .focus_visible(wide_focus_ring(&p))
            } else {
                s
            }
        });
        let media = self.media.map(|media| {
            node()
                .style(move || {
                    let s = style()
                        .direction(Direction::Row)
                        .align_items(Align::Center)
                        .justify_content(Justify::Center)
                        .shrink(0)
                        .gap(8);
                    let s = if has_description {
                        s.align_self(Align::Start).translate(0, 2)
                    } else {
                        s
                    };
                    if media_variant == ItemMediaVariant::Image {
                        let side = if size.get() == ItemSize::Sm {
                            32.0
                        } else {
                            40.0
                        };
                        s.size(side, side).overflow(Overflow::Hidden)
                    } else {
                        s
                    }
                })
                .child(media)
        });
        let content = column((
            text(self.title).style(style().font_size(12).weight(500).single_line()),
            self.description.map(|description| {
                text(description).style(
                    style()
                        .line_height(1.625)
                        .line_clamp(2)
                        .color(crate::ColorName::MutedForeground),
                )
            }),
        ))
        .style(style().grow(1).gap(4).min_width(0));
        let actions = self
            .actions
            .map(|actions| row(actions).style(style().align_items(Align::Center).gap(8).shrink(0)));
        let band = |view: kivact::view::View| {
            let view = view.into_view();
            row(view).style(
                style()
                    .width(pct(100))
                    .align_items(Align::Center)
                    .justify_content(Justify::SpaceBetween)
                    .gap(8),
            )
        };
        let header = self.header.map(band);
        let footer = self.footer.map(band);
        root.child(header)
            .child(media)
            .child(content)
            .child(actions)
            .child(footer)
            .into_view()
    }
}

/// A vertical list of [`item`]s with 16 px between them (`Role::List`).
#[track_caller]
pub fn item_group(children: impl IntoView) -> impl kivact::view::Component {
    column(children)
        .with_node(|root| root.role(Role::List))
        .style(style().width(pct(100)).gap(16))
}

/// A horizontal rule between items, with 8 px above and below.
#[track_caller]
pub fn item_separator() -> impl kivact::view::Component {
    node()
        .with_node(|root| root.role(Role::Separator))
        .orientation(Orientation::Horizontal)
        .style(
            style()
                .width(pct(100))
                .height(1)
                .shrink(0)
                .margin_y(8)
                .background(crate::ColorName::Border),
        )
}

kivact::state::constant_value!(ItemSize);
