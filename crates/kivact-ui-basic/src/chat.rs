//! Role: Chat surfaces: the Lyra bubble, bubble group and message row.

use crate::parts::pressable;
use crate::theme_access::tokens;
use crate::tokens::Tokens;
use kivact::state::{Callback, IntoValue};
use kivact::view::{Node, NodeBuilder, column, node, row};

use kivact::style::{Align, Color, Cursor, Direction, Overflow, Position, pct, style};
use kivact::view::{IntoView, Role, View};

/// The colors of a [`Bubble`] (bubble.tsx `variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BubbleVariant {
    /// Solid primary: the sender's own text.
    #[default]
    Default,
    /// Secondary fill.
    Secondary,
    /// Muted fill.
    Muted,
    /// A faint gray derived from the primary color.
    Tinted,
    /// Bordered on the background.
    Outline,
    /// No fill, border or padding.
    Ghost,
    /// Tinted destructive.
    Destructive,
}

/// The side a [`Bubble`] or [`Message`] sticks to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BubbleAlign {
    /// The leading side.
    #[default]
    Start,
    /// The trailing side.
    End,
}

/// The edge of a [`Bubble`] its reactions overlap.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BubbleReactionsSide {
    /// Over the top edge.
    Top,
    /// Over the bottom edge.
    #[default]
    Bottom,
}

/// The side a [`Message`] sticks to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum MessageAlign {
    /// The leading side: received messages.
    #[default]
    Start,
    /// The trailing side: sent messages.
    End,
}

/// The room a bubble keeps for its reactions: three quarters of a pill
/// (12 px text on a 16 px line and 2 px padding, `translate-y-3/4`) and its
/// 2 px ring.
const REACTIONS_OVERHANG: f64 = 18.0;

/// A chat bubble (bubble.tsx). Created by [`bubble`].
pub struct Bubble {
    root: Node,
    content: View,
    variant: BubbleVariant,
    align: kivact::state::Value<BubbleAlign>,
    reactions: Option<kivact::view::View>,
    reactions_side: kivact::state::Value<BubbleReactionsSide>,
    on_press: Option<Callback<()>>,
}

/// A bubble around `content` (usually text).
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn reply() -> impl IntoView {
///     basic::bubble("Sounds good, see you then.")
///         .variant(basic::BubbleVariant::Secondary)
///         .align(basic::BubbleAlign::End)
/// }
/// ```
#[track_caller]
pub fn bubble(content: impl IntoView) -> Bubble {
    Bubble {
        root: node(),
        content: content.into_view(),
        variant: BubbleVariant::Default,
        align: BubbleAlign::Start.into_value(),
        reactions: None,
        reactions_side: BubbleReactionsSide::Bottom.into_value(),
        on_press: None,
    }
}

impl Bubble {
    /// Choose the colors.
    pub fn variant(mut self, variant: BubbleVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Stick the bubble to the start or the end of its container.
    pub fn align(mut self, align: impl IntoValue<BubbleAlign>) -> Self {
        self.align = align.into_value();
        self
    }

    /// Show `reactions` overlapping the bubble's edge (`BubbleReactions`): a
    /// muted pill of emoji and counts (`text("👍")`, `text("+2")`) or of
    /// small ghost buttons. Three quarters of the pill hang out of the
    /// bubble, and the bubble keeps that much room on that side (a margin),
    /// so the pill does not cover the next message.
    pub fn reactions(mut self, reactions: impl IntoView) -> Self {
        self.reactions = Some(reactions.into_view());
        self
    }

    /// Choose the edge the reactions overlap; the bottom by default.
    pub fn reactions_side(mut self, side: impl IntoValue<BubbleReactionsSide>) -> Self {
        self.reactions_side = side.into_value();
        self
    }

    /// Make the bubble a button that reports presses to `f`.
    pub fn on_press(mut self, f: impl Into<kivact::state::Callback<()>>) -> Self {
        self.on_press = Some(f.into());
        self
    }
}

impl NodeBuilder for Bubble {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// `oklch(from primary 0.93 ..)` and friends for the neutral primary: the grays of lightness
/// 0.93, 0.88 (light) and 0.30, 0.35 (dark).
fn tinted(p: &Tokens, hovered: bool) -> Color {
    match (p.is_dark(), hovered) {
        (false, false) => Color::from_rgba_f64(0.910, 0.910, 0.910, 1.0),
        (false, true) => Color::from_rgba_f64(0.843, 0.843, 0.843, 1.0),
        (true, false) => Color::from_rgba_f64(0.180, 0.180, 0.180, 1.0),
        (true, true) => Color::from_rgba_f64(0.227, 0.227, 0.227, 1.0),
    }
}

/// Fill, text and border colors of a bubble body; `hovered` only applies to pressable bubbles.
fn colors(p: &Tokens, variant: BubbleVariant, hovered: bool) -> (Color, Color, Color) {
    let clear = Color::TRANSPARENT;
    let dark = p.is_dark();
    match variant {
        BubbleVariant::Default => (
            p.primary.with_alpha(if hovered { 0.8 } else { 1.0 }),
            p.primary_foreground,
            clear,
        ),
        BubbleVariant::Secondary => (
            if hovered {
                p.secondary.mix(p.foreground, 0.05)
            } else {
                p.secondary
            },
            p.secondary_foreground,
            clear,
        ),
        BubbleVariant::Muted => (
            if hovered {
                p.muted.mix(p.foreground, 0.05)
            } else {
                p.muted
            },
            p.foreground,
            clear,
        ),
        BubbleVariant::Tinted => (tinted(p, hovered), p.foreground, clear),
        BubbleVariant::Outline => (
            match (hovered, dark) {
                (false, _) => p.background,
                (true, false) => p.muted,
                (true, true) => p.input.with_alpha(0.3),
            },
            p.foreground,
            p.border,
        ),
        BubbleVariant::Ghost => (
            if hovered {
                p.muted.with_alpha(if dark { 0.5 } else { 1.0 })
            } else {
                clear
            },
            p.foreground,
            clear,
        ),
        BubbleVariant::Destructive => {
            let (base, over) = if dark { (0.2, 0.3) } else { (0.1, 0.2) };
            (
                p.destructive.with_alpha(if hovered { over } else { base }),
                p.destructive,
                clear,
            )
        }
    }
}

impl IntoView for Bubble {
    fn into_view(self) -> View {
        let variant = self.variant;
        let align = self.align;
        let end = move || align.get() == BubbleAlign::End;
        let actionable = self.on_press.is_some();
        let (body, _) = pressable(node(), Role::Button, self.on_press);
        let body = body
            .style(move || {
                let p = tokens();
                let (fill, fg, line) = colors(&p, variant, false);
                let s = style()
                    .direction(Direction::Column)
                    .align_self(if end() { Align::End } else { Align::Start })
                    .max_width(pct(100))
                    .min_width(0)
                    .border(1, line)
                    .radius(0)
                    .background(fill)
                    .color(fg)
                    .font_size(12)
                    .line_height(1.625)
                    .overflow(Overflow::Hidden);
                let s = if variant == BubbleVariant::Ghost {
                    s
                } else {
                    s.padding_x(10).padding_y(8)
                };
                if actionable {
                    let (fill, fg, line) = colors(&p, variant, true);
                    s.cursor(Cursor::Pointer)
                        .hover(style().background(fill).color(fg).border_color(line))
                        .focus_visible(crate::theme_access::focus_ring(&p))
                } else {
                    s
                }
            })
            .child(self.content);
        let side = self.reactions_side;
        let has_reactions = self.reactions.is_some();
        let hanging = move || has_reactions.then(|| side.get());
        let reactions = self.reactions.map(|reactions| {
            row(reactions).style(move || {
                let p = tokens();
                let s = style()
                    .position(Position::Absolute)
                    .align_items(Align::Center)
                    .gap(4)
                    .padding_x(6)
                    .padding_y(2)
                    .background(p.muted)
                    .color(p.foreground)
                    .font_size(12)
                    .ring(2, p.card)
                    .z_index(10);
                let s = match side.get() {
                    BubbleReactionsSide::Top => s.top(0).translate(0, pct(-75)),
                    BubbleReactionsSide::Bottom => s.bottom(0).translate(0, pct(75)),
                };
                if end() { s.right(12) } else { s.left(12) }
            })
        });
        // The reactions hang out of the bubble; the margin keeps them off
        // the next row. shadcn leaves that room to the caller.
        let root_style = move || {
            (match hanging() {
                Some(BubbleReactionsSide::Top) => style().margin_top(REACTIONS_OVERHANG),
                Some(BubbleReactionsSide::Bottom) => style().margin_bottom(REACTIONS_OVERHANG),
                None => style(),
            }
            .direction(Direction::Column)
            .position(Position::Relative)
            .align_self(if end() { Align::End } else { Align::Start })
            .max_width(if variant == BubbleVariant::Ghost {
                pct(100)
            } else {
                pct(80)
            })
            .min_width(0)
            .gap(4))
        };
        self.root
            .base_style(root_style)
            .child(body)
            .child(reactions)
            .into_view()
    }
}

/// Bubbles stacked with 8 px between them.
#[track_caller]
pub fn bubble_group(children: impl IntoView) -> impl kivact::view::Component {
    column(children).style(style().min_width(0).gap(8))
}

/// A chat message row (message.tsx). Created by [`message`].
pub struct Message {
    root: Node,
    content: View,
    align: kivact::state::Value<MessageAlign>,
    avatar: Option<kivact::view::View>,
    header: Option<kivact::view::View>,
    footer: Option<kivact::view::View>,
}

/// A message whose body is `content` (bubbles, text or any views).
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn received() -> impl IntoView {
///     basic::message(basic::bubble("Hi there!").variant(basic::BubbleVariant::Secondary))
///         .avatar(basic::avatar("AB"))
///         .header(text("Ada"))
///         .footer(text("10:42"))
/// }
/// ```
#[track_caller]
pub fn message(content: impl IntoView) -> Message {
    Message {
        root: node(),
        content: content.into_view(),
        align: MessageAlign::Start.into_value(),
        avatar: None,
        header: None,
        footer: None,
    }
}

impl Message {
    /// Put the message at the start (received) or end (sent) of the row.
    pub fn align(mut self, align: impl IntoValue<MessageAlign>) -> Self {
        self.align = align.into_value();
        self
    }

    /// Show `avatar` beside the content, aligned to its bottom.
    pub fn avatar(mut self, avatar: impl IntoView) -> Self {
        self.avatar = Some(avatar.into_view());
        self
    }

    /// Show `header` (a name) above the content, muted and medium.
    pub fn header(mut self, header: impl IntoView) -> Self {
        self.header = Some(header.into_view());
        self
    }

    /// Show `footer` (a time or status) below the content, muted and medium.
    pub fn footer(mut self, footer: impl IntoView) -> Self {
        self.footer = Some(footer.into_view());
        self
    }
}

impl NodeBuilder for Message {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Message {
    fn into_view(self) -> View {
        let align = self.align;
        let end = move || align.get() == MessageAlign::End;
        let has_footer = self.footer.is_some();
        let band = move |view: kivact::view::View| {
            let view = view.into_view();
            row(view).style(move || {
                style()
                    .align_self(Align::Stretch)
                    .align_items(Align::Center)
                    .justify_content(if end() {
                        kivact::style::Justify::End
                    } else {
                        kivact::style::Justify::Start
                    })
                    .max_width(pct(100))
                    .padding_x(10)
                    .font_size(12)
                    .weight(500)
                    .color(tokens().muted_foreground)
            })
        };
        let body =
            column((self.header.map(band), self.content, self.footer.map(band))).style(move || {
                style().grow(1).min_width(0).gap(8).align_items(if end() {
                    Align::End
                } else {
                    Align::Start
                })
            });
        let avatar = self.avatar.map(|avatar| {
            row(avatar).style(move || {
                let s = style()
                    .min_width(32)
                    .shrink(0)
                    .align_self(Align::End)
                    .align_items(Align::Center)
                    .radius(999)
                    .overflow(Overflow::Hidden)
                    .background(tokens().muted);
                if has_footer { s.translate(0, -32) } else { s }
            })
        });
        self.root
            .base_style(move || {
                style()
                    .direction(if end() {
                        Direction::RowReverse
                    } else {
                        Direction::Row
                    })
                    .width(pct(100))
                    .min_width(0)
                    .gap(6)
                    .font_size(12)
            })
            .child(avatar)
            .child(body)
            .into_view()
    }
}

/// Messages stacked with 6 px between them.
#[track_caller]
pub fn message_group(children: impl IntoView) -> impl kivact::view::Component {
    column(children).style(style().min_width(0).gap(6))
}

kivact::state::constant_value!(BubbleAlign);

kivact::state::constant_value!(MessageAlign);

kivact::state::constant_value!(BubbleReactionsSide);
