//! Role: Draw the console's button, form control and field recipes.
use crate::color::{LINE_STRONG, ON_DARK, PRIMARY_WEAK, SURFACE_3, TEXT_2, TEXT_3};
use crate::control_parts::{button_style, skew};
use crate::size::CONTROL_HEIGHT;
use kivact::draw::{Layer, LineCap, LineJoin, Path, Stroke, TextFormat};
use kivact::geometry::{Point, Rect};
use kivact::style::color::{ACCENT, ACCENT_TEXT, DANGER, SURFACE, TEXT, TEXT_MUTED};
use kivact::style::{Align, FlexDirection, Style, Theme, Transform, pct, style};
use kivact::ui::ButtonVariant;
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

pub(crate) fn button(_: &Theme, state: &Look<ButtonState>, _base: Style) -> Style {
    button_style(
        state.variant::<ButtonVariant>().unwrap_or_default(),
        state.size,
        state.busy,
        state.variant::<Slant>().is_some(),
    )
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
