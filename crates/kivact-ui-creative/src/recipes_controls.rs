//! Role: Creative appearance of the standard buttons, toggles, fields and sliders.
//! The radio groups, splitters, progress bars and disclosures too.
use crate::color;
use crate::fonts::mono_family;
use crate::recipe_parts::{
    SPLITTER_THICKNESS, button_style, check_style, field_style, segmented_option_style,
    segmented_style, splitter_look,
};
use crate::size::ROW;
use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::geometry::{Point, Rect, Size};
use kivact::style::{Align, Cursor, FlexDirection, Justify, Position, Style, Theme, pct, style};
use kivact::ui::theme::{
    AxisState, ButtonState, ButtonTheme, CheckableState, CheckboxTheme, DisclosureRecipeState,
    DisclosureTheme, Look, ProgressState, ProgressTheme, RadioGroupTheme, SliderTheme,
    SplitterTheme, SwitchTheme, TextAreaTheme, TextInputState, TextInputTheme, ToggleButtonState,
    ToggleButtonTheme,
};
use kivact::ui::{ButtonVariant, ControlSize, ProgressVariant, RadioGroupVariant};
use kivact::view::Orientation;

/// The theme of a standard button in the kit's look.
pub(crate) fn button() -> ButtonTheme {
    ButtonTheme::default()
        .root(button_root)
        .content(button_content)
}

fn button_root(t: &Theme, s: &Look<ButtonState>, _base: Style) -> Style {
    button_look(t, s)
}

fn button_look(t: &Theme, s: &Look<ButtonState>) -> Style {
    let base = button_style(t);
    let base = match s.variant::<ButtonVariant>().unwrap_or_default() {
        ButtonVariant::Primary => base
            .background(color::ACCENT)
            .color(color::ACCENT_INK)
            .font_weight(600)
            .hover(
                style()
                    .background(color::ACCENT_HIGH)
                    .color(color::ACCENT_INK),
            ),
        ButtonVariant::Danger => base.hover(
            style()
                .background(t.get(&color::X_VAR).with_alpha(0.16))
                .color(color::X),
        ),
        ButtonVariant::Ghost | ButtonVariant::Link => base
            .background(color::TRANSPARENT_INK)
            .border(0, color::HAIR)
            .color(color::MUTED)
            .hover(
                style()
                    .background(color::MENU_HOVER)
                    .color(color::TEXT_HIGH),
            ),
        _ => base,
    };
    let base = match s.size {
        ControlSize::ExtraSmall => base.height(16).font_size(10).border_radius(5).padding_x(6),
        ControlSize::Small => base.height(20).font_size(11).border_radius(6),
        ControlSize::Medium => base.height(ROW),
        ControlSize::Large => base.height(30).font_size(13).border_radius(9).padding_x(18),
    };
    // A square toolbar action: as wide as the button is high.
    if s.is_icon_only() {
        base.aspect_ratio(1.0).padding(0)
    } else {
        base
    }
}

fn button_content(_t: &Theme, _s: &Look<ButtonState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(6)
}

/// The theme of a toggle button: the secondary button, in the selection
/// colors while pressed.
pub(crate) fn toggle_button() -> ToggleButtonTheme {
    ToggleButtonTheme::default().root(toggle_button_root)
}

fn toggle_button_root(t: &Theme, s: &Look<ToggleButtonState>, _base: Style) -> Style {
    toggle_button_look(t, s)
}

/// The look of a toggle button; a kit draws its own toggles with it.
pub(crate) fn toggle_button_look(t: &Theme, s: &Look<ToggleButtonState>) -> Style {
    let mut state = ButtonState::new(s.input, s.size);
    state.leading_icon = s.leading_icon;
    state.label_hidden = s.label_hidden;
    let base = button_look(t, &Look::new(state).with_variant(ButtonVariant::Secondary));
    if s.pressed {
        base.background(color::SELECTION)
            .color(color::ACCENT_HIGH)
            .border(1, t.get(&color::ACCENT_VAR).with_alpha(0.35))
    } else {
        base
    }
}

/// The theme of a checkbox: a row with a box, a check or a dash.
pub(crate) fn checkbox() -> CheckboxTheme {
    CheckboxTheme::default().root(checkbox_root)
}

fn checkbox_root(_t: &Theme, s: &Look<CheckableState>, _base: Style) -> Style {
    toggle(**s, false)
}

/// The theme of a switch: a track with a thumb.
pub(crate) fn switch() -> SwitchTheme {
    SwitchTheme::default().root(switch_root)
}

fn switch_root(_t: &Theme, s: &Look<CheckableState>, _base: Style) -> Style {
    toggle(**s, true)
}

fn toggle(state: CheckableState, switch: bool) -> Style {
    check_style()
        .padding_left(if switch { 37 } else { 21 })
        .paint(move |d, size| {
            let on = state.is_checked();
            let mixed = state.is_mixed();
            let y = (size.height - 16.) / 2.;
            if switch {
                d.rounded_rect(
                    Rect::new(0., y, 30., 16.),
                    8.,
                    if on { color::ACCENT } else { color::STRONG },
                );
                d.rounded_rect(
                    Rect::new(if on { 16. } else { 2. }, y + 2., 12., 12.),
                    6.,
                    if on { color::ACCENT_INK } else { color::MUTED },
                );
            } else {
                d.rounded_rect(
                    Rect::new(0.5, y + 1.5, 13., 13.),
                    4.,
                    if on || mixed {
                        color::ACCENT
                    } else {
                        color::STRONG
                    },
                );
                if mixed {
                    d.line(
                        Point::new(3.5, y + 8.),
                        Point::new(10.5, y + 8.),
                        1.7,
                        color::ACCENT_INK,
                    );
                } else if on {
                    d.stroke_path(
                        &Path::new()
                            .move_to(Point::new(3., y + 8.))
                            .line_to(Point::new(6., y + 10.5))
                            .line_to(Point::new(11., y + 5.5)),
                        &Stroke::new(1.7).cap(LineCap::Round).join(LineJoin::Round),
                        color::ACCENT_INK,
                    );
                }
            }
        })
}

/// The theme of a single-line text field.
pub(crate) fn text_input() -> TextInputTheme {
    TextInputTheme::default().root(text_input_root)
}

fn text_input_root(_t: &Theme, _s: &Look<TextInputState>, _base: Style) -> Style {
    field_style().padding_y(0).min_height(ROW)
}

/// The theme of a multi-line text field.
pub(crate) fn text_area() -> TextAreaTheme {
    TextAreaTheme::default().root(text_area_root)
}

fn text_area_root(_t: &Theme, _s: &Look<TextInputState>, _base: Style) -> Style {
    field_style().height(100).min_height(ROW)
}

/// The radio group: a column or a wrapping row, or a segmented control.
pub(crate) fn radio_group() -> RadioGroupTheme {
    RadioGroupTheme::default()
        .group(radio_group_layout)
        .item(radio_item)
        .indicator(radio_indicator)
        .description(radio_description)
}

fn radio_group_layout(_t: &Theme, orientation: &Look<Orientation>, _base: Style) -> Style {
    if orientation.variant::<RadioGroupVariant>() == Some(RadioGroupVariant::Segmented) {
        return segmented_style();
    }
    let s = style().gap(8);
    match **orientation {
        Orientation::Vertical => s.flex_direction(FlexDirection::Column),
        _ => s.flex_direction(FlexDirection::Row).flex_wrap(true).gap(12),
    }
}

/// A radio row: the indicator beside the label.
fn radio_item(t: &Theme, state: &Look<CheckableState>, _base: Style) -> Style {
    if state.variant::<RadioGroupVariant>() == Some(RadioGroupVariant::Segmented) {
        return segmented_option_style(t);
    }
    check_style()
}

/// The 14 px disc of an option: the accent around the theme's `dot`, in the
/// accent's ink, while chosen.
fn radio_indicator(_t: &Theme, s: &Look<CheckableState>, _base: Style) -> Style {
    if s.variant::<RadioGroupVariant>() == Some(RadioGroupVariant::Segmented) {
        // A segment shows its choice by its fill, without a disc.
        return style().position(Position::Absolute).size(0, 0).clip(true);
    }
    let on = s.is_checked();
    style()
        .size(14, 14)
        .flex_shrink(0)
        .border_radius(7)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .background(if on { color::ACCENT } else { color::STRONG })
        .color(color::ACCENT_INK)
}

fn radio_description(_t: &Theme, _s: &Look<CheckableState>, _base: Style) -> Style {
    style().font_size(10.5).color(color::MUTED)
}

/// The thickness of a slider's bar and the side of its thumb.
const BAR: f64 = 4.0;
const THUMB: f64 = 16.0;

/// The slider: a bar, its filled range and a round thumb.
pub(crate) fn slider() -> SliderTheme {
    SliderTheme::default()
        .track(slider_track)
        .range(slider_range)
        .thumb(slider_thumb)
}

/// The bar of a track or of its filled range, across the middle of `size`.
fn bar(size: Size) -> Rect {
    if size.width >= size.height {
        Rect::new(0.0, (size.height - BAR) / 2.0, size.width, BAR)
    } else {
        Rect::new((size.width - BAR) / 2.0, 0.0, BAR, size.height)
    }
}

fn slider_track(_t: &Theme, s: &Look<AxisState>, _base: Style) -> Style {
    let base = style()
        .flex_shrink(0)
        .cursor(Cursor::EwResize)
        .paint(move |d, size| d.rounded_rect(bar(size), BAR / 2.0, color::STRONG));
    match s.orientation {
        Orientation::Vertical => base.width(THUMB).height(160).cursor(Cursor::NsResize),
        _ => base.height(24).min_width(80),
    }
}

fn slider_range(_t: &Theme, _s: &Look<AxisState>, _base: Style) -> Style {
    style().paint(move |d, size| {
        if size.width > 0.0 && size.height > 0.0 {
            d.rounded_rect(bar(size), BAR / 2.0, color::ACCENT);
        }
    })
}

/// A 16 px disc in the high text color, like the range's endpoints.
fn slider_thumb(_t: &Theme, _s: &Look<AxisState>, _base: Style) -> Style {
    style()
        .size(THUMB, THUMB)
        .border_radius(THUMB / 2.0)
        .background(color::TEXT_HIGH)
}

/// The splitter: a transparent bar, accent while hovered or focused.
pub(crate) fn splitter() -> SplitterTheme {
    SplitterTheme::default().handle(splitter_handle)
}

fn splitter_handle(_t: &Theme, s: &Look<AxisState>, _base: Style) -> Style {
    let thickness = SPLITTER_THICKNESS;
    let look = splitter_look();
    match s.orientation {
        Orientation::Vertical => look
            .height(thickness)
            .width(pct(100))
            .flex_shrink(0)
            .cursor(Cursor::NsResize),
        _ => look
            .width(thickness)
            .height(pct(100))
            .flex_shrink(0)
            .cursor(Cursor::EwResize),
    }
}

/// The progress bar: a field-colored track filled with the accent from the
/// shown fraction; a ring or a gauge keeps its own look.
pub(crate) fn progress() -> ProgressTheme {
    ProgressTheme::default()
        .track(progress_track)
        .fill(progress_fill)
        .label(progress_label)
}

fn progress_track(_t: &Theme, s: &Look<ProgressState>, base: Style) -> Style {
    match s.variant::<ProgressVariant>() {
        Some(ProgressVariant::Ring | ProgressVariant::Gauge) => base,
        _ => style()
            .height(8)
            .width(pct(100))
            .border_radius(4)
            .clip(true)
            .background(color::FIELD),
    }
}

fn progress_fill(_t: &Theme, look: &Look<ProgressState>, base: Style) -> Style {
    match look.variant::<ProgressVariant>() {
        Some(ProgressVariant::Ring | ProgressVariant::Gauge) => base,
        _ => {
            let state = **look;
            style()
                .width(pct(100))
                .height(pct(100))
                .paint(move |d, size| {
                    let (x, width) = if state.value.is_some() {
                        (0.0, size.width * state.shown.get().clamp(0.0, 1.0))
                    } else {
                        let (start, length) = state.sweep();
                        (size.width * start, size.width * length)
                    };
                    d.rounded_rect(Rect::new(x, 0.0, width, size.height), 4.0, color::ACCENT);
                })
        }
    }
}

fn progress_label(t: &Theme, _s: &Look<ProgressState>, base: Style) -> Style {
    base.font_family(mono_family(t)).font_size(10.5)
}

/// The theme of a disclosure: an island whose header row starts with the chevron.
pub(crate) fn disclosure() -> DisclosureTheme {
    DisclosureTheme::default()
        .root(disclosure_root)
        .trigger(disclosure_trigger)
        .indicator(disclosure_indicator)
        .content(disclosure_content)
}

fn disclosure_root(_t: &Theme, _s: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .border_radius(7)
        .background(color::ISLAND)
        .border(1, color::HAIR)
}

/// The chevron is the trigger's last child, drawn first by the reversed row.
fn disclosure_trigger(_t: &Theme, _s: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::RowReverse)
        .height(30)
        .gap(6)
        .padding_x(8)
        .align_items(Align::Center)
        .background(color::HEADER)
        .hover(style().background(kivact::style::color::HOVER))
}

fn disclosure_indicator(_t: &Theme, state: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    style()
        .size(16, 16)
        .flex_shrink(0)
        .rotate(if state.open { 0.0 } else { -90.0 })
}

fn disclosure_content(_t: &Theme, _s: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    style()
        .enter_size(Orientation::Vertical)
        .exit_size(Orientation::Vertical)
        .flex_direction(FlexDirection::Column)
        .padding(10)
}

/// Install the controls' recipes into `theme`.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(button())
        .with(toggle_button())
        .with(checkbox())
        .with(switch())
        .with(text_input())
        .with(text_area())
        .with(radio_group())
        .with(slider())
        .with(splitter())
        .with(progress())
        .with(disclosure())
}
