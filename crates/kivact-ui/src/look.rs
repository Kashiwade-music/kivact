//! Role: The default recipes: the core's look, made only of the theme's tokens.
//!
//! These are the defaults of [`UiRecipes`](super::UiRecipes) for the button,
//! checkbox, switch and text fields; `look_extra` and `look_controls` hold
//! those of the multi-part components. They follow the recipe rules of
//! `super::recipes`: state looks as state styles, the checkbox and switch
//! drawn with local `Style::paint`, and the text fields style only (a native
//! editing leaf has no paint). The default `Link` reads the hover state to
//! underline, the one thing a state style cannot set.

use kivact_core::draw::{CornerRadius, LineCap, LineJoin, Stroke};
use kivact_core::draw::{Draw, Path, Point, Rect, Size};
use kivact_core::style::{
    Align, Background, Color, ColorToken, Direction, Justify, Style, TextDecorationLine, Theme,
    style,
};

use super::values::{
    ButtonState, ButtonVariant, CheckState, ControlSize, TextInputState, ToggleButtonState,
};
use super::variants::Look;
use kivact_interaction::CheckedState;

/// The font size of a control.
fn font_size(theme: &Theme, size: ControlSize) -> f32 {
    match size {
        ControlSize::ExtraSmall | ControlSize::Small => theme.text().size_small,
        ControlSize::Medium => theme.text().size,
        ControlSize::Large => theme.text().size + 2.0,
    }
}

/// The height of a control: the theme's `control_height`, 80% of it for
/// a small one and 65% for an extra small one.
pub(super) fn height(theme: &Theme, size: ControlSize) -> f32 {
    match size {
        ControlSize::ExtraSmall => (theme.control_height() * 0.65).round(),
        ControlSize::Small => (theme.control_height() * 0.8).round(),
        ControlSize::Medium => theme.control_height(),
        ControlSize::Large => theme.control_height() + 8.0,
    }
}

/// The gap between a control's parts.
fn gap(theme: &Theme, size: ControlSize) -> f32 {
    match size {
        ControlSize::ExtraSmall => theme.space().xs,
        ControlSize::Small => theme.space().sm,
        ControlSize::Medium | ControlSize::Large => theme.space().md,
    }
}

/// A tint of the theme over `base`, as one background of two layers.
fn tinted(tint: ColorToken, base: impl Into<Color>) -> Background {
    (Color::token(tint), base.into()).into()
}

/// The button recipe: a row of an optional icon and the label.
pub(super) fn button(theme: &Theme, state: &Look<ButtonState>) -> Style {
    let size = state.size;
    let variant = state
        .variant::<ButtonVariant>()
        .copied()
        .unwrap_or_default();
    let s = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(gap(theme, size))
        .height(height(theme, size))
        .padding_x(match size {
            ControlSize::ExtraSmall => theme.space().sm,
            ControlSize::Small => theme.space().md,
            ControlSize::Medium | ControlSize::Large => theme.space().lg,
        })
        .radius(theme.radius().md)
        .font_size(font_size(theme, size))
        .weight(theme.text().weight_strong)
        .border(1, Color::TRANSPARENT);
    // An icon button is a square of the button's height.
    let s = if state.icon_only {
        s.width(height(theme, size)).padding_x(0)
    } else {
        s
    };
    match variant {
        ButtonVariant::Primary | ButtonVariant::Danger => {
            let fill = if variant == ButtonVariant::Primary {
                ColorToken::Accent
            } else {
                ColorToken::Danger
            };
            s.background(fill)
                .color(ColorToken::AccentText)
                .hover(style().background(tinted(ColorToken::Hover, fill)))
                .pressed(style().background(tinted(ColorToken::Pressed, fill)))
        }
        ButtonVariant::Secondary => s
            .background(ColorToken::Surface)
            .color(ColorToken::Text)
            .border_color(ColorToken::Border)
            .hover(style().background(tinted(ColorToken::Hover, ColorToken::Surface)))
            .pressed(style().background(tinted(ColorToken::Pressed, ColorToken::Surface))),
        ButtonVariant::Ghost => s
            .color(ColorToken::Text)
            .hover(style().background(ColorToken::Hover))
            .pressed(style().background(ColorToken::Pressed)),
        ButtonVariant::Outline => s
            .background(ColorToken::Surface)
            .color(ColorToken::Text)
            .border_color(ColorToken::Border)
            .hover(style().background(ColorToken::Hover))
            .pressed(style().background(ColorToken::Pressed)),
        ButtonVariant::Link => {
            let s = s.color(ColorToken::Accent).text_underline_offset(4);
            // The label inherits the root's text decoration; the actual hover
            // state underlines it, as CSS `hover:underline` (a text
            // decoration is not a state style property).
            if state.input.hovered {
                s.text_decoration(TextDecorationLine::UNDERLINE)
            } else {
                s
            }
        }
    }
}

/// The side of a check box and the height of a switch's track.
fn indicator(theme: &Theme, size: ControlSize) -> f32 {
    font_size(theme, size).round() + 2.0
}

/// The room a toggle keeps at its left edge for its drawing.
fn leading(theme: &Theme) -> f32 {
    theme.space().sm
}

/// The width of the drawn part of a toggle.
fn indicator_width(theme: &Theme, size: ControlSize, switch: bool) -> f32 {
    let side = indicator(theme, size);
    if switch { (side * 1.8).round() } else { side }
}

/// The root of a check box or a switch: a row whose left edge keeps room
/// for the drawing of the recipe.
fn toggle(theme: &Theme, size: ControlSize, switch: bool) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .min_height(height(theme, size))
        .padding_left(leading(theme) + indicator_width(theme, size, switch) + gap(theme, size))
        .padding_right(theme.space().sm)
        .radius(theme.radius().md)
        .font_size(font_size(theme, size))
        .color(ColorToken::Text)
        .hover(style().background(ColorToken::Hover))
        .pressed(style().background(ColorToken::Pressed))
}

pub(super) fn checkbox(theme: &Theme, look: &CheckState) -> Style {
    let (theme, state) = (theme.clone(), *look);
    toggle(&theme, state.size, false).paint(move |d, size| draw_checkbox(&theme, &state, d, size))
}

pub(super) fn switch(theme: &Theme, look: &CheckState) -> Style {
    let (theme, state) = (theme.clone(), *look);
    toggle(&theme, state.size, true).paint(move |d, size| draw_switch(&theme, &state, d, size))
}

/// The box of a check box: filled with the accent and a check mark (or a
/// bar when mixed) while on, an outlined box otherwise.
pub(super) fn draw_checkbox(theme: &Theme, look: &CheckState, d: &mut Draw, size: Size) {
    let side = indicator(theme, look.size);
    let r = theme.radius().sm;
    let rect = Rect::new(leading(theme), (size.height - side) / 2.0, side, side);
    if look.checked != CheckedState::Unchecked {
        d.rounded_rect(rect, r, ColorToken::Accent);
        let at = |x: f32, y: f32| Point::new(rect.x + side * x, rect.y + side * y);
        let mark = Stroke::new(2.0).cap(LineCap::Round).join(LineJoin::Round);
        if look.checked == CheckedState::Indeterminate {
            d.polyline(
                [at(0.25, 0.5), at(0.75, 0.5)],
                &mark,
                ColorToken::AccentText,
            );
        } else {
            d.polyline(
                [at(0.25, 0.52), at(0.43, 0.7), at(0.76, 0.3)],
                &mark,
                ColorToken::AccentText,
            );
        }
    } else {
        d.rounded_rect(rect, r, ColorToken::Surface);
        let inner = Rect::new(rect.x + 0.5, rect.y + 0.5, side - 1.0, side - 1.0);
        d.stroke_path(
            &Path::new().rounded_rect(inner, [CornerRadius::from(r); 4]),
            &Stroke::new(1.0),
            if look.invalid {
                ColorToken::Danger
            } else {
                ColorToken::Border
            },
        );
    }
}

/// The track and thumb of a switch.
pub(super) fn draw_switch(theme: &Theme, look: &CheckState, d: &mut Draw, size: Size) {
    let on = look.checked.is_checked();
    let h = indicator(theme, look.size);
    let w = indicator_width(theme, look.size, true);
    let track = Rect::new(leading(theme), (size.height - h) / 2.0, w, h);
    d.rounded_rect(
        track,
        h / 2.0,
        if on {
            ColorToken::Accent
        } else {
            ColorToken::Border
        },
    );
    let inset = 2.0;
    let thumb = h - 2.0 * inset;
    let x = if on {
        track.x + w - inset - thumb
    } else {
        track.x + inset
    };
    d.rounded_rect(
        Rect::new(x, track.y + inset, thumb, thumb),
        thumb / 2.0,
        if on {
            ColorToken::AccentText
        } else {
            ColorToken::Surface
        },
    );
}

/// The toggle button recipe: a row of an optional icon and the label,
/// filled with the theme's pressed color while it is on.
pub(super) fn toggle_button(theme: &Theme, state: &ToggleButtonState) -> Style {
    let size = state.size;
    let s = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(gap(theme, size))
        .height(height(theme, size))
        .min_width(height(theme, size))
        .padding_x(match size {
            ControlSize::ExtraSmall => theme.space().sm,
            ControlSize::Small => theme.space().md,
            ControlSize::Medium | ControlSize::Large => theme.space().lg,
        })
        .radius(theme.radius().md)
        .font_size(font_size(theme, size))
        .weight(theme.text().weight_strong)
        .border(1, Color::TRANSPARENT)
        .color(ColorToken::Text)
        .hover(style().background(ColorToken::Hover))
        .pressed(style().background(ColorToken::Pressed));
    let s = if state.icon_only {
        s.width(height(theme, size)).padding_x(0)
    } else {
        s
    };
    if state.pressed {
        s.background(ColorToken::Pressed)
            .border_color(ColorToken::Border)
    } else {
        s
    }
}

/// The text input recipe: a bordered field that rings on focus.
pub(super) fn text_input(theme: &Theme, state: &TextInputState) -> Style {
    let size = state.size;
    let s = style()
        .height(height(theme, size))
        .min_height(0)
        .padding_x(theme.space().md)
        .radius(theme.radius().md)
        .border(1, ColorToken::Border)
        .background(ColorToken::Surface)
        .color(ColorToken::Text)
        .font_size(font_size(theme, size))
        // A field shows its focus however it got it, as a browser's does.
        .focus(style().border_color(ColorToken::Focus).ring(
            theme.focus_ring(),
            Color::token(ColorToken::Focus).with_alpha(0.35),
        ));
    if state.invalid {
        s.border_color(ColorToken::Danger)
    } else {
        s
    }
}

pub(super) fn text_area(theme: &Theme, state: &TextInputState) -> Style {
    text_input(theme, state)
        .height(100)
        .padding_y(theme.space().sm)
}
pub(super) fn button_content(theme: &Theme, state: &ButtonState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(gap(theme, state.size))
}
/// The icon box of a button: the icon at its own size, centered.
pub(super) fn button_icon(_: &Theme, _: &ButtonState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .shrink(0)
}
/// A three-quarter ring the button turns while busy.
pub(super) fn button_busy(_: &Theme, state: &Look<ButtonState>) -> Style {
    let color = match state
        .variant::<ButtonVariant>()
        .copied()
        .unwrap_or_default()
    {
        ButtonVariant::Primary | ButtonVariant::Danger => ColorToken::AccentText,
        _ => ColorToken::Text,
    };
    style().size(14, 14).shrink(0).paint(move |d, size| {
        let center = Point::new(size.width / 2.0, size.height / 2.0);
        let radius = size.width.min(size.height) / 2.0 - 1.5;
        let arc = Path::new().arc(center, radius, 0.0, 270.0, false);
        d.stroke_path(&arc, &Stroke::new(2.0).cap(LineCap::Round), color);
    })
}
