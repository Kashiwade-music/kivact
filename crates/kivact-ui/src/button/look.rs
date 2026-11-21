//! Role: The button recipe state, tokens, recipe table and default looks.
use crate::look_common::{font_size, gap, height};
use crate::recipe::recipe_table;

use crate::control_size::ControlSize;
use crate::variants::Look;
use kivact_core::custom::PartState;
use kivact_core::draw::{LineCap, Path, Stroke};
use kivact_core::geometry::Point;
use kivact_core::style::{
    Align, Background, Color, FlexDirection, Justify, Style, TextDecorationLine, Theme, style,
};
/// A tint of the theme over `base`, as one background of two layers.
pub(crate) fn tinted(tint: Color, base: impl Into<Color>) -> Background {
    (tint, base.into()).into()
}

/// The button recipe: a row of an optional icon and the label.
pub(crate) fn button(theme: &Theme, state: &Look<ButtonState>) -> Style {
    let size = state.size;
    let variant = state
        .variant::<ButtonVariant>()
        .copied()
        .unwrap_or_default();
    let s = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(gap(theme, size))
        .height(height(theme, size))
        .padding_x(match size {
            ControlSize::ExtraSmall => theme.get(&kivact_core::style::space::SM_VAR),
            ControlSize::Small => theme.get(&kivact_core::style::space::MD_VAR),
            ControlSize::Medium | ControlSize::Large => {
                theme.get(&kivact_core::style::space::LG_VAR)
            }
        })
        .border_radius(theme.get(&kivact_core::style::radius::MD_VAR))
        .font_size(font_size(theme, size))
        .font_weight(theme.get(&kivact_core::style::font::WEIGHT_STRONG_VAR))
        .border(1, Color::TRANSPARENT);
    // An icon button is a square of the button's height.
    let s = if state.is_icon_only() {
        s.width(height(theme, size)).padding_x(0)
    } else {
        s
    };
    match variant {
        ButtonVariant::Primary | ButtonVariant::Danger => {
            let fill = if variant == ButtonVariant::Primary {
                kivact_core::style::color::ACCENT
            } else {
                kivact_core::style::color::DANGER
            };
            s.background(fill)
                .color(kivact_core::style::color::ACCENT_TEXT)
                .hover(style().background(tinted(kivact_core::style::color::HOVER, fill)))
                .active(style().background(tinted(kivact_core::style::color::PRESSED, fill)))
        }
        ButtonVariant::Secondary => s
            .background(kivact_core::style::color::SURFACE)
            .color(kivact_core::style::color::TEXT)
            .border_color(kivact_core::style::color::BORDER)
            .hover(style().background(tinted(
                kivact_core::style::color::HOVER,
                kivact_core::style::color::SURFACE,
            )))
            .active(style().background(tinted(
                kivact_core::style::color::PRESSED,
                kivact_core::style::color::SURFACE,
            ))),
        ButtonVariant::Ghost => s
            .color(kivact_core::style::color::TEXT)
            .hover(style().background(kivact_core::style::color::HOVER))
            .active(style().background(kivact_core::style::color::PRESSED)),
        ButtonVariant::Outline => s
            .background(kivact_core::style::color::SURFACE)
            .color(kivact_core::style::color::TEXT)
            .border_color(kivact_core::style::color::BORDER)
            .hover(style().background(kivact_core::style::color::HOVER))
            .active(style().background(kivact_core::style::color::PRESSED)),
        ButtonVariant::Link => {
            let s = s
                .color(kivact_core::style::color::ACCENT)
                .text_underline_offset(4);
            // The label inherits the root's text decoration; the actual hover
            // state underlines it, as CSS `hover:underline` (a text
            // decoration is not a state style property).
            if state.input.is_hovered() {
                s.text_decoration(TextDecorationLine::UNDERLINE)
            } else {
                s
            }
        }
    }
}

pub(crate) fn button_content(theme: &Theme, state: &Look<ButtonState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(gap(theme, state.size))
}

/// The icon box of a button: the icon at its own size, centered.
pub(crate) fn button_icon(_: &Theme, _: &Look<ButtonState>) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .flex_shrink(0)
}

/// A three-quarter ring the button turns while busy.
pub(crate) fn button_busy(_: &Theme, state: &Look<ButtonState>) -> Style {
    let color = match state
        .variant::<ButtonVariant>()
        .copied()
        .unwrap_or_default()
    {
        ButtonVariant::Primary | ButtonVariant::Danger => kivact_core::style::color::ACCENT_TEXT,
        _ => kivact_core::style::color::TEXT,
    };
    style().size(14, 14).flex_shrink(0).paint(move |d, size| {
        let center = Point::new(size.width / 2.0, size.height / 2.0);
        let radius = size.width.min(size.height) / 2.0 - 1.5;
        let arc = Path::new().arc(center, radius, 0.0, 270.0, false);
        d.stroke_path(&arc, &Stroke::new(2.0).cap(LineCap::Round), color);
    })
}

/// What a button's recipes know of it: its root, its content row, its
/// icon boxes and its busy indicator get the same state.
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonState {
    /// The root's actual input state. Hover, press and focus looks are
    /// `style().hover(..)`, `.active(..)` and `.focus_visible(..)` in the
    /// returned style; read this only for what a state style cannot change
    /// (the default `Link` underlines its label while hovered: a text
    /// decoration is not a state style property).
    pub input: PartState,
    pub size: ControlSize,
    pub busy: bool,
    /// Whether the button opens something and whether that is open: `None`
    /// for a plain button, `Some(open)` when the button publishes an
    /// expanded state (a popup, menu or disclosure attached its trigger to
    /// it, or the caller set `.expanded(..)`). A look draws an open
    /// trigger as such (shadcn's `aria-expanded:bg-muted`) and does not
    /// nudge a popup trigger while pressed.
    pub expanded: Option<bool>,
    /// Whether the button shows a leading icon ([`Button::icon`](super::Button::icon)).
    pub leading_icon: bool,
    /// Whether the button shows a trailing icon
    /// ([`Button::trailing`](super::Button::trailing)).
    pub trailing_icon: bool,
    /// Whether the label is hidden ([`Button::hide_label`](super::Button::hide_label));
    /// it stays the accessible name.
    pub label_hidden: bool,
}

impl ButtonState {
    /// Whether only the leading icon shows: the button has one and its
    /// label is hidden. A look draws it as a square of the button's height.
    pub fn is_icon_only(&self) -> bool {
        self.leading_icon && self.label_hidden
    }

    /// The state of a plain, idle button: for a component that draws one of
    /// its own parts as the theme's button (a kit's toggle, say) by calling
    /// the theme's [`ButtonTheme::root`](crate::theme::ButtonTheme) recipe with
    /// [`Look::new`](super::Look::new) with the button's variant added by
    /// [`Look::with_variant`](super::Look::with_variant).
    pub fn new(input: PartState, size: ControlSize) -> Self {
        ButtonState {
            input,
            size,
            busy: false,
            expanded: None,
            leading_icon: false,
            trailing_icon: false,
            label_hidden: false,
        }
    }
}

/// What a button does, which its recipe draws (shadcn's `variant`).
///
/// These are meanings every theme draws; a kit's own looks (a tone, a
/// shape) are further [`Variant`](super::Variant) values:
/// [`Button::variant`](super::Button::variant) takes this and a kit's type
/// alike, and a recipe reads each with
/// [`Look::variant`](super::Look::variant) (a button without a
/// `ButtonVariant` is the default, `Primary`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ButtonVariant {
    /// The main action of its area, filled with the accent.
    #[default]
    Primary,
    /// A secondary action on the surface.
    Secondary,
    /// A bordered button on the surface for a secondary action beside a
    /// primary one; the default look is a 1px border on the surface.
    Outline,
    /// A button with no fill until it is hovered, for toolbars and quiet actions.
    Ghost,
    /// An action that destroys or removes something.
    Danger,
    /// A text-only button in the accent color that reads as an inline link;
    /// the default look underlines it only while hovered.
    Link,
}

impl crate::variants::Variant for ButtonVariant {}

recipe_table! {
    /// The recipes of a button: its root, its content row, its busy indicator and its icon box.
    ButtonTheme {
        root(root_recipe): ButtonState = button,
        content(content_recipe): ButtonState = button_content,
        busy(busy_recipe): ButtonState = button_busy,
        icon(icon_recipe): ButtonState = button_icon,
    }
}
