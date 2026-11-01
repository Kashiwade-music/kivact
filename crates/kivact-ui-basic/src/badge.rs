//! Role: The Lyra badge: a small status label that can act as a link.

use crate::parts::{pressable, sized_icon};
use crate::theme_access::{tokens, wide_focus_ring};
use crate::tokens::Tokens;
use kivact::control::{Node, NodeBuilder, node, text};
use kivact::primitive::NodeExt;
use kivact::primitive::PartState;
use kivact::style::{Align, Color, Cursor, Justify, TextDecorationLine, style};
use kivact::view::HandlerResult;
use kivact::view::{IntoText, IntoView, Role, View, callback};
use kivact::{Callback, Value};

/// The look of a [`Badge`] (badge.tsx `variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BadgeVariant {
    /// Solid primary.
    #[default]
    Default,
    /// Secondary fill.
    Secondary,
    /// Tinted destructive.
    Destructive,
    /// Bordered, no fill.
    Outline,
    /// No fill; muted on hover.
    Ghost,
    /// Primary text, underlined on hover.
    Link,
}

/// A badge (badge.tsx). Created by [`badge`].
pub struct Badge {
    root: Node,
    label: Value<String>,
    variant: BadgeVariant,
    icon: Option<View>,
    trailing_icon: Option<View>,
    on_press: Option<Callback<()>>,
}

/// A badge showing `text`.
///
/// ```no_run
/// use kivact::control::icon;
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn status() -> impl IntoView {
///     basic::badge("Verified")
///         .variant(basic::BadgeVariant::Secondary)
///         .icon(icon("badge-check"))
/// }
/// ```
#[track_caller]
pub fn badge(text_value: impl IntoText) -> Badge {
    let label = text_value.into_text_value();
    Badge {
        root: node().name(label),
        label,
        variant: BadgeVariant::Default,
        icon: None,
        trailing_icon: None,
        on_press: None,
    }
}

impl Badge {
    /// Choose the look.
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Show `icon` before the text; it is drawn at 12 px.
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }

    /// Show `icon` after the text; it is drawn at 12 px.
    pub fn trailing_icon(mut self, icon: impl IntoView) -> Self {
        self.trailing_icon = Some(icon.into_view());
        self
    }

    /// Make the badge a link-like button that reports presses to `f`.
    pub fn on_press<R: HandlerResult>(mut self, f: impl Fn() -> R + 'static) -> Self {
        self.on_press = Some(callback(move |_| f()));
        self
    }
}

impl NodeBuilder for Badge {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// Background, text and border colors of `variant`; hover colors apply to pressable badges.
/// The fill, text and border colors of `variant`, at rest or while a
/// pressable badge is hovered.
fn colors(p: &Tokens, variant: BadgeVariant, hovered: bool) -> (Color, Color, Color) {
    let dark = p.is_dark();
    let clear = Color::TRANSPARENT;
    match variant {
        BadgeVariant::Default => (
            p.primary.with_alpha(if hovered { 0.8 } else { 1.0 }),
            p.primary_foreground,
            clear,
        ),
        BadgeVariant::Secondary => (
            p.secondary.with_alpha(if hovered { 0.8 } else { 1.0 }),
            p.secondary_foreground,
            clear,
        ),
        BadgeVariant::Destructive => {
            let (base, over) = if dark { (0.2, 0.3) } else { (0.1, 0.2) };
            let alpha = if hovered { over } else { base };
            (p.destructive.with_alpha(alpha), p.destructive, clear)
        }
        BadgeVariant::Outline => {
            if hovered {
                (p.muted, p.muted_foreground, p.border)
            } else {
                (clear, p.foreground, p.border)
            }
        }
        BadgeVariant::Ghost => {
            if hovered {
                (
                    p.muted.with_alpha(if dark { 0.5 } else { 1.0 }),
                    p.muted_foreground,
                    clear,
                )
            } else {
                (clear, p.foreground, clear)
            }
        }
        BadgeVariant::Link => (clear, p.primary, clear),
    }
}

/// badge.tsx: the variant's colors, and while pressable the hover colors,
/// the underline of a hovered link (a text decoration, which a state style
/// cannot change, so it follows `input`) and the wide focus ring.
fn badge_style(
    p: &Tokens,
    variant: BadgeVariant,
    pressable: bool,
    input: &PartState,
    padding: (f32, f32),
) -> kivact::style::Style {
    let (fill, fg, line) = colors(p, variant, false);
    let mut s = style()
        .direction(kivact::style::Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .shrink(0)
        .gap(4)
        .height(20)
        .padding_left(padding.0)
        .padding_right(padding.1)
        .radius(0)
        .border(1, line)
        .background(fill)
        .color(fg)
        .font_size(12)
        .weight(500)
        .text_wrap(false)
        .clip(true);
    if variant == BadgeVariant::Link && input.hovered {
        s = s
            .text_decoration(TextDecorationLine::UNDERLINE)
            .text_underline_offset(4);
    }
    if !pressable {
        return s;
    }
    let (fill, fg, line) = colors(p, variant, true);
    let focus = if variant == BadgeVariant::Destructive {
        style().border_color(p.destructive.with_alpha(0.4)).ring(
            3,
            p.destructive
                .with_alpha(if p.is_dark() { 0.4 } else { 0.2 }),
        )
    } else {
        wide_focus_ring(p)
    };
    s.cursor(Cursor::Pointer)
        .hover(style().background(fill).color(fg).border_color(line))
        .focus_visible(focus)
}

impl IntoView for Badge {
    fn into_view(self) -> View {
        let variant = self.variant;
        let actionable = self.on_press.is_some();
        let (root, state) = pressable(self.root, Role::Button, self.on_press);
        let padding = (
            if self.icon.is_some() { 6.0 } else { 8.0 },
            if self.trailing_icon.is_some() {
                6.0
            } else {
                8.0
            },
        );
        root.base_style(move || badge_style(&tokens(), variant, actionable, &state.get(), padding))
            .child(self.icon.map(|icon| sized_icon(icon, 12.0)))
            .child(text(self.label))
            .child(self.trailing_icon.map(|icon| sized_icon(icon, 12.0)))
            .into_view()
    }
}
