//! Role: Draw the console's button, form control and field recipes.
use crate::color::{
    DARK_SHADOW, LINE_STRONG, NAVY, ON_DARK, PRIMARY_HOVER, PRIMARY_STRONG, PRIMARY_WEAK,
    SIDEBAR_ALT, SURFACE_3, TEXT_2, TEXT_3, YELLOW,
};
use crate::size::CONTROL_HEIGHT;
use kivact::draw::{Layer, LineCap, LineJoin, Path, Stroke, TextFormat};
use kivact::geometry::{Point, Rect};
use kivact::style::color::{ACCENT, ACCENT_TEXT, DANGER, SURFACE, TEXT, TEXT_MUTED};
use kivact::style::{
    Align, Color, FlexDirection, Justify, Shadow, Style, Theme, Transform, pct, style,
};
use kivact::ui::ButtonVariant;
use kivact::ui::ControlSize;
use kivact::ui::Slant;
use kivact::ui::theme::{
    ButtonState, ButtonTheme, CheckableState, CheckboxTheme, FieldState, FieldTheme, Look,
    SwitchTheme, TextAreaTheme, TextInputState, TextInputTheme, ToggleButtonState,
    ToggleButtonTheme,
};

/// Install the button, toggle, checkbox, switch, text, field recipes, then
/// the choice and progress recipes.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme
        .with(
            ButtonTheme::default()
                .root(button)
                .content(button_content)
                .busy(button_busy),
        )
        .with(ToggleButtonTheme::default().root(toggle))
        .with(CheckboxTheme::default().root(checkbox))
        .with(SwitchTheme::default().root(switch))
        .with(TextInputTheme::default().root(text_input))
        .with(TextAreaTheme::default().root(text_area))
        .with(
            FieldTheme::default()
                .root(field_root)
                .label(field_label)
                .hint(field_hint)
                .error(field_error),
        );
    let theme = crate::recipes_controls_choice::install(theme);
    crate::recipes_controls_progress::install(theme)
}

/// The skew of a slanted button's root.
pub(crate) fn skew(slant: bool) -> f64 {
    if slant { -12.0 } else { 0.0 }
}

pub(crate) fn button(_: &Theme, state: &Look<ButtonState>, _base: Style) -> Style {
    button_style(
        state.variant::<ButtonVariant>().unwrap_or_default(),
        state.size,
        state.busy,
        state.variant::<Slant>().is_some(),
    )
}

/// The look of a button of `variant` and `size`, slanted or not, busy or
/// not: what a segment of the console's radio group draws too.
pub(crate) fn button_style(
    variant: ButtonVariant,
    size: ControlSize,
    loading: bool,
    slant: bool,
) -> Style {
    let skew = skew(slant);
    let (bg, fg, hover, shadow) = match variant {
        ButtonVariant::Primary => (ACCENT, ACCENT_TEXT, PRIMARY_HOVER, PRIMARY_STRONG),
        ButtonVariant::Secondary => (NAVY, ACCENT_TEXT, SIDEBAR_ALT, DARK_SHADOW),
        ButtonVariant::Outline => (SURFACE, ACCENT, PRIMARY_WEAK, PRIMARY_WEAK),
        ButtonVariant::Ghost | ButtonVariant::Link => {
            (Color::TRANSPARENT, TEXT_2, PRIMARY_WEAK, Color::TRANSPARENT)
        }
        ButtonVariant::Danger => (DANGER, ACCENT_TEXT, DANGER, DANGER),
    };
    let mut s = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .background(bg)
        .color(fg)
        .font_size(13)
        .font_weight(700)
        .text_wrap(false)
        .transform(Transform::new().skew_x(skew))
        .shadow(Shadow::new(0, 2, 0, shadow))
        .hover(style().background(hover))
        .active(style().transform(Transform::new().skew_x(skew).translate_y(2)));
    if variant == ButtonVariant::Outline {
        s = s.border(1.5, ACCENT);
    }
    s = s.merge(&match size {
        ControlSize::ExtraSmall => style()
            .height(CONTROL_HEIGHT - 12.)
            .padding_x(8)
            .font_size(11),
        ControlSize::Small => style()
            .height(CONTROL_HEIGHT - 6.)
            .padding_x(12)
            .font_size(12),
        ControlSize::Medium => style().height(CONTROL_HEIGHT).padding_x(18).font_size(13),
        ControlSize::Large => style()
            .height(CONTROL_HEIGHT + 8.)
            .padding_x(28)
            .font_size(14),
    });
    if loading {
        s = s.disabled(style().opacity(1.));
    }
    if variant == ButtonVariant::Primary {
        s = s.paint(|d, s| d.fill_rect(Rect::new(s.width - 10., 5., 5., 5.), YELLOW));
    }
    s
}

pub(crate) fn button_content(_: &Theme, state: &Look<ButtonState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .gap(8)
        .align_items(Align::Center)
        .opacity(if state.busy { 0. } else { 1. })
        .transform(Transform::new().skew_x(-skew(state.variant::<Slant>().is_some())))
}

/// The arc the button turns while busy.
pub(crate) fn button_busy(_: &Theme, state: &Look<ButtonState>, _base: Style) -> Style {
    let variant = state.variant::<ButtonVariant>().unwrap_or_default();
    style()
        .position(kivact::style::Position::Absolute)
        .left(pct(50) - 7)
        .top(pct(50) - 7)
        .size(14, 14)
        .paint(move |d, _| {
            let color = match variant {
                ButtonVariant::Outline => ACCENT,
                ButtonVariant::Ghost | ButtonVariant::Link => TEXT_2,
                _ => ACCENT_TEXT,
            };
            d.stroke_path(
                &Path::new().arc(Point::new(7., 7.), 6., 0., 270., false),
                &Stroke::new(2.),
                color,
            );
        })
}

pub(crate) fn toggle(_: &Theme, state: &Look<ToggleButtonState>, _base: Style) -> Style {
    let s = button_style(ButtonVariant::Outline, state.size, false, false);
    if state.pressed {
        s.background(ACCENT).color(ACCENT_TEXT)
    } else {
        s
    }
}

pub(crate) fn text_input(_: &Theme, _: &Look<TextInputState>, _base: Style) -> Style {
    style()
        .height(CONTROL_HEIGHT)
        .min_height(CONTROL_HEIGHT)
        .padding_x(12)
        .padding_y(0)
        .border_radius(0)
        .border(1, LINE_STRONG)
        .background(SURFACE)
        .color(TEXT)
        .hover(style().border_color(TEXT_3))
        .focus(style().border_color(ACCENT).outline(3, PRIMARY_WEAK))
        .disabled(style().background(SURFACE_3).color(TEXT_3).opacity(1.))
}

pub(crate) fn text_area(_: &Theme, _: &Look<TextInputState>, _base: Style) -> Style {
    style()
        .min_height(88)
        .padding_x(12)
        .padding_y(8)
        .border_radius(0)
        .line_height(1.6)
        .border(1, LINE_STRONG)
        .focus(style().border_color(ACCENT).outline(3, PRIMARY_WEAK))
}

pub(crate) fn checkbox(_: &Theme, state: &Look<CheckableState>, _base: Style) -> Style {
    let state: CheckableState = **state;
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .min_height(24)
        .padding_left(25)
        .paint(move |d, size| {
            d.layer(
                Layer::new().transform(Transform::new().translate_y((size.height - 16.) / 2.)),
                |d| {
                    let c = state.is_checked();
                    let mixed = state.is_mixed();
                    let primary = ACCENT;
                    d.fill_rect(
                        Rect::new(0., 0., 16., 16.),
                        if c || mixed { primary } else { SURFACE },
                    );
                    d.stroke_rect(
                        Rect::new(0.75, 0.75, 14.5, 14.5),
                        1.5,
                        if c || mixed { primary } else { LINE_STRONG },
                    );
                    if mixed {
                        d.line(Point::new(4., 8.), Point::new(12., 8.), 2., ACCENT_TEXT);
                    } else if c {
                        d.stroke_path(
                            &Path::new()
                                .move_to(Point::new(3., 8.))
                                .line_to(Point::new(6., 11.))
                                .line_to(Point::new(13., 4.)),
                            &Stroke::new(2.).cap(LineCap::Round).join(LineJoin::Round),
                            ACCENT_TEXT,
                        );
                    }
                },
            );
        })
}

pub(crate) fn switch(_: &Theme, state: &Look<CheckableState>, _base: Style) -> Style {
    let state: CheckableState = **state;
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .min_height(24)
        .padding_left(49)
        .paint(move |d, size| {
            d.layer(
                Layer::new().transform(Transform::new().translate_y((size.height - 20.) / 2.)),
                |d| {
                    let c = state.is_checked();
                    d.fill_rect(
                        Rect::new(0., 0., 40., 20.),
                        if c { ACCENT } else { LINE_STRONG },
                    );
                    d.fill_rect(Rect::new(if c { 23. } else { 3. }, 3., 14., 14.), ON_DARK);
                    d.text(
                        &state
                            .switch_text()
                            .map(|text| text.get())
                            .unwrap_or_default(),
                        Point::new(if c { 5. } else { 20. }, 6.),
                        TextFormat {
                            size: 7.5,
                            weight: 700,
                            color: ON_DARK,
                            ..Default::default()
                        },
                    );
                },
            );
        })
}

pub(crate) fn field_root(_: &Theme, _: &Look<FieldState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .width(pct(100))
        .min_width(0)
        .gap(6)
}

pub(crate) fn field_label(_: &Theme, state: &Look<FieldState>, _base: Style) -> Style {
    style()
        .font_size(11)
        .font_weight(700)
        .line_height(1.)
        .color(if state.invalid { DANGER } else { TEXT_MUTED })
}

pub(crate) fn field_hint(_: &Theme, _: &Look<FieldState>, _base: Style) -> Style {
    style().font_size(11.5).color(TEXT_MUTED)
}

pub(crate) fn field_error(_: &Theme, _: &Look<FieldState>, _base: Style) -> Style {
    style().font_size(11.5).color(DANGER)
}

/// The diamond outline filling the `side` square at `(x, y)`, inset by `inset`.
pub(crate) fn diamond(x: f64, y: f64, side: f64, inset: f64) -> Path {
    let (lo, mid, hi) = (inset, side / 2., side - inset);
    Path::new()
        .move_to(Point::new(x + mid, y + lo))
        .line_to(Point::new(x + hi, y + mid))
        .line_to(Point::new(x + mid, y + hi))
        .line_to(Point::new(x + lo, y + mid))
        .close()
}
